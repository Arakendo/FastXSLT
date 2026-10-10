//! Copy a guarded single-effective-source union using the existing node copier.

use super::{
    ApplySelection, ExecutionFailure, Instruction, InvocationControl, NodeId, ResultNode,
    RuntimeVariables, SequenceInputs, copy_source_node, required_source_context, source_unions,
};

pub(super) fn execute(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let Instruction::CopyOfXslt10MixedUnion {
        select,
        recover_unattached_attributes,
        location,
    } = instruction
    else {
        unreachable!("copy union dispatch receives its compiled instruction")
    };
    let ApplySelection::Xslt10MixedUnion(alternatives) = select else {
        unreachable!("compiled mixed copy unions retain the shared mixed selection plan")
    };
    let outcome = (|| {
        let (source, context) = required_source_context(inputs, context)?;
        // Finish selection (including every foreign empty-branch guard) before
        // constructing any result-owned payload. No partial copy escapes failure.
        let selected =
            source_unions::select_mixed(inputs, alternatives, context, variables, control)?;
        let mut result = Vec::new();
        for node in selected {
            result.extend(copy_source_node(
                source,
                inputs.request_id,
                node,
                *recover_unattached_attributes,
                control,
            )?);
        }
        Ok(result)
    })();
    outcome.map_err(|mut failure: ExecutionFailure| {
        if failure.location.is_none() {
            failure.location = Some(location.clone());
        }
        failure
    })
}
