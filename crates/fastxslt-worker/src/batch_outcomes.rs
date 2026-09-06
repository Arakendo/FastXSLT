//! Private bounded outcome retention and framing for AR-0021 batches.

use std::{io, io::Write, mem::size_of};

use fastxslt::workbench::{ExperimentalEngine, WorkbenchCancellation, WorkbenchFailure};

use super::{
    BATCH_RESULT, BatchMemberCommand, ERROR, MAX_BATCH_RESPONSE_BYTES, RESULT, checked_wire_add,
    encoded_failure_bytes, encoded_string_bytes, write_byte, write_failure, write_failure_fields,
    write_string, write_u32,
};

pub(super) type BatchOutcome = (String, Result<String, WorkbenchFailure>);

pub(super) fn collect(
    engine: &ExperimentalEngine,
    members: Vec<BatchMemberCommand>,
) -> Result<(Vec<BatchOutcome>, usize), WorkbenchFailure> {
    let mut outcomes = Vec::with_capacity(members.len());
    let mut response_bytes = 1_usize + size_of::<u32>();
    execute_and_retain(engine, members, &mut outcomes, &mut response_bytes)?;
    Ok((outcomes, response_bytes))
}

pub(super) fn execute_and_retain(
    engine: &ExperimentalEngine,
    members: Vec<BatchMemberCommand>,
    outcomes: &mut Vec<BatchOutcome>,
    response_bytes: &mut usize,
) -> Result<(), WorkbenchFailure> {
    for member in members {
        retain(outcomes, response_bytes, execute_member(engine, member))?;
    }
    Ok(())
}

pub(super) fn retain(
    outcomes: &mut Vec<BatchOutcome>,
    response_bytes: &mut usize,
    outcome: BatchOutcome,
) -> Result<(), WorkbenchFailure> {
    let outcome_bytes = encoded_bytes(&outcome.0, &outcome.1, "batch response size")
        .map_err(|error| limit_failure(error.to_string()))?;
    let next_bytes = checked_wire_add(*response_bytes, outcome_bytes, "batch response size")
        .map_err(|error| limit_failure(error.to_string()))?;
    if next_bytes > MAX_BATCH_RESPONSE_BYTES {
        return Err(limit_failure(format!(
            "batch response bytes exceed {MAX_BATCH_RESPONSE_BYTES}"
        )));
    }
    *response_bytes = next_bytes;
    outcomes.push(outcome);
    Ok(())
}

pub(super) fn execute_member(
    engine: &ExperimentalEngine,
    member: BatchMemberCommand,
) -> BatchOutcome {
    let cancellation = WorkbenchCancellation::new();
    if member.cancelled {
        cancellation.cancel();
    }
    let result = member.maximum_xslt_instructions.map_or_else(
        || engine.transform_with_cancellation(&member.request_id, cancellation),
        |maximum| engine.transform_with_xslt_instruction_limit(&member.request_id, maximum),
    );
    (member.request_id, result)
}

pub(super) fn encoded_bytes(
    request_id: &str,
    result: &Result<String, WorkbenchFailure>,
    context: &str,
) -> io::Result<usize> {
    let mut outcome_bytes = 1_usize;
    match result {
        Ok(result) => {
            outcome_bytes =
                checked_wire_add(outcome_bytes, encoded_string_bytes(request_id)?, context)?;
            outcome_bytes =
                checked_wire_add(outcome_bytes, encoded_string_bytes(result)?, context)?;
        }
        Err(failure) => {
            outcome_bytes =
                checked_wire_add(outcome_bytes, encoded_failure_bytes(failure)?, context)?;
        }
    }
    Ok(outcome_bytes)
}

pub(super) fn write_fields(
    output: &mut impl Write,
    request_id: &str,
    result: Result<String, WorkbenchFailure>,
) -> io::Result<()> {
    match result {
        Ok(result) => {
            write_byte(output, RESULT)?;
            write_string(output, request_id)?;
            write_string(output, &result)
        }
        Err(failure) => {
            write_byte(output, ERROR)?;
            write_failure_fields(output, &failure)
        }
    }
}

pub(super) fn write_all(output: &mut impl Write, outcomes: Vec<BatchOutcome>) -> io::Result<()> {
    let mut response_bytes = 1_usize + size_of::<u32>();
    for (request_id, result) in &outcomes {
        let outcome_bytes = encoded_bytes(request_id, result, "batch response size")?;
        response_bytes = checked_wire_add(response_bytes, outcome_bytes, "batch response size")?;
        if response_bytes > MAX_BATCH_RESPONSE_BYTES {
            return write_failure(
                output,
                &limit_failure(format!(
                    "batch response bytes exceed {MAX_BATCH_RESPONSE_BYTES}"
                )),
            );
        }
    }

    write_byte(output, BATCH_RESULT)?;
    write_u32(
        output,
        u32::try_from(outcomes.len()).expect("bounded batch count fits u32"),
    )?;
    for (request_id, result) in outcomes {
        write_fields(output, &request_id, result)?;
    }
    output.flush()
}

fn limit_failure(detail: String) -> WorkbenchFailure {
    WorkbenchFailure {
        code: "FXWB1004".to_owned(),
        category: "limit".to_owned(),
        request_id: None,
        location: None,
        detail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_rejects_before_retaining_an_over_limit_outcome() {
        let mut outcomes = Vec::new();
        let mut response_bytes = 1 + size_of::<u32>();
        for index in 0..6 {
            retain(
                &mut outcomes,
                &mut response_bytes,
                (format!("member-{index}"), Ok("a".repeat(165_011))),
            )
            .expect("six representative large outcomes fit");
        }
        let retained_bytes = response_bytes;

        let failure = retain(
            &mut outcomes,
            &mut response_bytes,
            ("member-6".to_owned(), Ok("b".repeat(165_011))),
        )
        .expect_err("seventh representative large outcome exceeds aggregate retention");

        assert_eq!(failure.code, "FXWB1004");
        assert_eq!(failure.category, "limit");
        assert_eq!(outcomes.len(), 6);
        assert_eq!(response_bytes, retained_bytes);
        assert!(response_bytes <= MAX_BATCH_RESPONSE_BYTES);
    }
}
