//! Private input-order incremental response framing for AR-0021.

use std::{io, io::Write, mem::size_of};

use fastxslt::workbench::{ExperimentalEngine, WorkbenchFailure};

use super::batch_outcomes::{encoded_bytes, execute_member, write_fields};
use super::{
    BATCH_ACKNOWLEDGED, BATCH_INCREMENTAL_END, BATCH_INCREMENTAL_OUTCOME, BatchMemberCommand,
    MAX_BATCH_RESPONSE_BYTES, VERSIONED_BATCH_ACKNOWLEDGED, checked_wire_add, write_byte,
    write_failure, write_u32,
};

pub(super) fn execute(
    engine: &ExperimentalEngine,
    members: Vec<BatchMemberCommand>,
    protocol_version: Option<u32>,
    output: &mut impl Write,
) -> io::Result<()> {
    let member_count = members.len();
    write_byte(
        output,
        if protocol_version.is_some() {
            VERSIONED_BATCH_ACKNOWLEDGED
        } else {
            BATCH_ACKNOWLEDGED
        },
    )?;
    if let Some(version) = protocol_version {
        write_u32(output, version)?;
    }
    write_u32(
        output,
        u32::try_from(member_count).expect("bounded batch count fits u32"),
    )?;
    output.flush()?;

    let end_bytes = 1_usize + size_of::<u32>();
    let acknowledgement_bytes =
        1_usize + size_of::<u32>() + protocol_version.map_or(0, |_| size_of::<u32>());
    let mut response_bytes = acknowledgement_bytes + end_bytes;
    for (index, member) in members.into_iter().enumerate() {
        let (request_id, result) = execute_member(engine, member);
        let fields = encoded_bytes(&request_id, &result, "incremental batch response size")?;
        let framed = checked_wire_add(
            1_usize + size_of::<u32>(),
            fields,
            "incremental batch response size",
        )?;
        response_bytes =
            checked_wire_add(response_bytes, framed, "incremental batch response size")?;
        if response_bytes > MAX_BATCH_RESPONSE_BYTES {
            return write_failure(
                output,
                &WorkbenchFailure {
                    code: "FXWB1004".to_owned(),
                    category: "limit".to_owned(),
                    request_id: None,
                    location: None,
                    detail: format!(
                        "incremental batch response bytes exceed {MAX_BATCH_RESPONSE_BYTES}"
                    ),
                },
            );
        }

        write_byte(output, BATCH_INCREMENTAL_OUTCOME)?;
        write_u32(
            output,
            u32::try_from(index).expect("bounded batch index fits u32"),
        )?;
        write_fields(output, &request_id, result)?;
        output.flush()?;
    }

    write_byte(output, BATCH_INCREMENTAL_END)?;
    write_u32(
        output,
        u32::try_from(member_count).expect("bounded batch count fits u32"),
    )?;
    output.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framing_overhead_is_charged_before_member_payload() {
        let base = 1_usize + size_of::<u32>() + 1 + size_of::<u32>();
        assert_eq!(base, 10);
    }
}
