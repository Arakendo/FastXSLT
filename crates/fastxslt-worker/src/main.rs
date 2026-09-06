//! Length-prefixed isolated transport for the ASP.NET boundary workbench.

mod batch_outcomes;
mod incremental_batch;

use batch_outcomes::{
    BatchOutcome, collect as collect_batch_outcomes,
    execute_and_retain as execute_and_retain_batch_members, write_all as write_batch_outcomes,
};

use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    mem::size_of,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

use fastxslt::workbench::{
    ExperimentalEngine, WorkbenchCancellation, WorkbenchFailure, WorkbenchLimits,
    WorkbenchResource, WorkbenchStylesheetResources,
};

const INITIALIZE: u8 = 1;
const TRANSFORM: u8 = 2;
const SHUTDOWN: u8 = 3;
const NON_COOPERATING_PROBE: u8 = 4;
const CANCELLED_TRANSFORM: u8 = 5;
const CONTROLLED_TRANSFORM: u8 = 6;
const CANCEL: u8 = 7;
const UNPAUSED_CONTROLLED_TRANSFORM: u8 = 8;
const INSTRUCTION_LIMITED_TRANSFORM: u8 = 9;
const INITIALIZE_WITH_STYLESHEET_DEPENDENCY: u8 = 10;
const MEASURED_TRANSFORM: u8 = 11;
const TRANSFORM_BATCH: u8 = 12;
const CONTROLLED_TRANSFORM_BATCH: u8 = 13;
const BATCH_LOSS_PROBE: u8 = 14;
const BATCH_TRANSFER_LOSS_PROBE: u8 = 15;
const ACTIVE_CANCELLATION_BATCH: u8 = 16;
const NATURAL_CANCELLATION_BATCH: u8 = 17;
const INCREMENTAL_TRANSFORM_BATCH: u8 = 18;
const CONTROLLED_INCREMENTAL_TRANSFORM_BATCH: u8 = 19;
const VERSIONED_INCREMENTAL_TRANSFORM_BATCH: u8 = 20;
const INCREMENTAL_BATCH_PROTOCOL_VERSION: u32 = 1;
const READY: u8 = 0x81;
const RESULT: u8 = 0x82;
const STOPPED: u8 = 0x83;
const PROBE_STARTED: u8 = 0x84;
const TRANSFORM_STARTED: u8 = 0x85;
const MEASURED_RESULT: u8 = 0x86;
const BATCH_RESULT: u8 = 0x87;
const BATCH_ACKNOWLEDGED: u8 = 0x88;
const BATCH_MEMBER_STARTED: u8 = 0x89;
const BATCH_MEMBER_FINISHED: u8 = 0x8a;
const BATCH_INCREMENTAL_OUTCOME: u8 = 0x8b;
const BATCH_INCREMENTAL_END: u8 = 0x8c;
const VERSIONED_BATCH_ACKNOWLEDGED: u8 = 0x8d;
const ERROR: u8 = 0xff;
const MAX_IDENTITY_BYTES: usize = 4_096;
const MAX_RESOURCE_BYTES: usize = 1_048_576;
const EVENT_QUEUE_CAPACITY: usize = 1;
const MAX_BATCH_MEMBERS: usize = 128;
const MAX_BATCH_COMMAND_BYTES: usize = 1_048_576;
const MAX_BATCH_RESPONSE_BYTES: usize = 1_048_576;

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());
    let (events, incoming) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
    let reader_events = events.clone();
    std::thread::spawn(move || read_commands(&reader_events));
    let mut supervisor = Supervisor::new(events);

    while let Ok(event) = incoming.recv() {
        if supervisor.handle_event(event, &mut output)? == LoopControl::Stop {
            return Ok(());
        }
    }
    Ok(())
}

struct Supervisor {
    engine: Option<Arc<ExperimentalEngine>>,
    active: Option<ActiveInvocation>,
    stop_after_active: bool,
    events: mpsc::SyncSender<Event>,
}

impl Supervisor {
    fn new(events: mpsc::SyncSender<Event>) -> Self {
        Self {
            engine: None,
            active: None,
            stop_after_active: false,
            events,
        }
    }

    fn handle_event(&mut self, event: Event, output: &mut impl Write) -> io::Result<LoopControl> {
        match event {
            Event::Command(command) => self.handle_command(command, output),
            Event::Completed { request_id, result } => {
                if self.active.as_ref().map(|value| &value.request_id) == Some(&request_id) {
                    let active = self.active.take().expect("matching active invocation");
                    match active.completion {
                        ActiveCompletion::Single => {
                            write_transform_result(output, &request_id, result)?;
                        }
                        ActiveCompletion::Batch {
                            mut outcomes,
                            remaining,
                            mut response_bytes,
                        } => {
                            let target = (request_id, result);
                            let retained =
                                batch_outcomes::retain(&mut outcomes, &mut response_bytes, target)
                                    .and_then(|()| {
                                        let engine =
                                            self.engine.as_ref().expect("active batch has engine");
                                        execute_and_retain_batch_members(
                                            engine,
                                            remaining,
                                            &mut outcomes,
                                            &mut response_bytes,
                                        )
                                    });
                            match retained {
                                Ok(()) => write_batch_outcomes(output, outcomes)?,
                                Err(failure) => write_failure(output, &failure)?,
                            }
                        }
                    }
                    if self.stop_after_active {
                        write_byte(output, STOPPED)?;
                        output.flush()?;
                        return Ok(LoopControl::Stop);
                    }
                }
                Ok(LoopControl::Continue)
            }
            Event::InputClosed(result) => result.map(|()| LoopControl::Stop),
        }
    }

    fn handle_command(
        &mut self,
        command: Command,
        output: &mut impl Write,
    ) -> io::Result<LoopControl> {
        match command {
            Command::Initialize {
                source_id,
                source,
                stylesheet_id,
                stylesheet,
                stylesheet_resources,
            } => match ExperimentalEngine::new_with_stylesheet_resources(
                source_id,
                source,
                stylesheet_id,
                stylesheet,
                stylesheet_resources,
                WorkbenchLimits::default(),
            ) {
                Ok(initialized) => {
                    self.engine = Some(Arc::new(initialized));
                    write_byte(output, READY)?;
                    output.flush()?;
                }
                Err(failure) => write_failure(output, &failure)?,
            },
            Command::Transform(TransformCommand {
                request_id,
                cancelled,
                controlled,
                first_charge_barrier,
                maximum_xslt_instructions,
                measurement,
            }) => self.begin_transform(
                TransformCommand {
                    request_id,
                    cancelled,
                    controlled,
                    first_charge_barrier,
                    maximum_xslt_instructions,
                    measurement,
                },
                output,
            )?,
            Command::Cancel { request_id } => {
                if let Some(invocation) = &self.active
                    && invocation.request_id == request_id
                {
                    invocation.cancellation.cancel();
                }
            }
            Command::TransformBatch { members } => {
                self.transform_batch(members, output)?;
            }
            Command::IncrementalTransformBatch {
                members,
                protocol_version,
            } => {
                self.incremental_transform_batch(members, protocol_version, output)?;
            }
            Command::UnsupportedBatchProtocolVersion(version) => {
                write_unsupported_batch_protocol(version, output)?;
            }
            Command::BatchLossProbe {
                request_ids,
                park_at_index,
            } => self.batch_loss_probe(request_ids, park_at_index, output)?,
            Command::BatchTransferLossProbe {
                request_ids,
                truncate_at_index,
            } => self.batch_transfer_loss_probe(request_ids, truncate_at_index, output)?,
            Command::ActiveCancellationBatch {
                members,
                cancel_at_index,
                first_charge_barrier,
            } => self.begin_active_cancellation_batch(
                members,
                cancel_at_index,
                first_charge_barrier,
                output,
            )?,
            Command::Shutdown => {
                if let Some(invocation) = &self.active {
                    invocation.cancellation.cancel();
                    self.stop_after_active = true;
                } else {
                    write_byte(output, STOPPED)?;
                    output.flush()?;
                    return Ok(LoopControl::Stop);
                }
            }
            Command::NonCooperatingProbe { request_id } => {
                write_byte(output, PROBE_STARTED)?;
                write_string(output, &request_id)?;
                output.flush()?;
                loop {
                    std::thread::park();
                }
            }
            Command::Unknown(operation) => write_unknown_operation(operation, output)?,
        }
        Ok(LoopControl::Continue)
    }

    fn begin_transform(
        &mut self,
        command: TransformCommand,
        output: &mut impl Write,
    ) -> io::Result<()> {
        let TransformCommand {
            request_id,
            cancelled,
            controlled,
            first_charge_barrier,
            maximum_xslt_instructions,
            measurement,
        } = command;
        let queue_elapsed = measurement
            .as_ref()
            .map_or(Duration::ZERO, |value| value.decoded_at.elapsed());
        if self.active.is_some() {
            return write_failure(
                output,
                &worker_failure(
                    "FXWB1003",
                    Some(request_id),
                    "worker already has an active invocation",
                ),
            );
        }
        let Some(engine) = &self.engine else {
            return write_failure(
                output,
                &worker_failure(
                    "FXWB1001",
                    Some(request_id),
                    "worker has not been initialized",
                ),
            );
        };
        let cancellation = WorkbenchCancellation::new();
        if cancelled {
            cancellation.cancel();
        }
        if !controlled {
            let execution_started = Instant::now();
            let result = maximum_xslt_instructions.map_or_else(
                || engine.transform_with_cancellation(&request_id, cancellation),
                |maximum| engine.transform_with_xslt_instruction_limit(&request_id, maximum),
            );
            let execution_elapsed = execution_started.elapsed();
            if let Some(measurement) = measurement {
                return write_measured_transform_result(
                    output,
                    &request_id,
                    result,
                    measurement.decode_elapsed,
                    queue_elapsed,
                    execution_elapsed,
                );
            }
            return write_transform_result(output, &request_id, result);
        }
        let cancellation = if first_charge_barrier {
            WorkbenchCancellation::with_first_charge_barrier()
        } else {
            WorkbenchCancellation::new()
        };
        start_transform(
            Arc::clone(engine),
            request_id.clone(),
            cancellation.clone(),
            self.events.clone(),
        );
        self.active = Some(ActiveInvocation {
            request_id: request_id.clone(),
            cancellation,
            completion: ActiveCompletion::Single,
        });
        if first_charge_barrier {
            while !self
                .active
                .as_ref()
                .is_some_and(|value| value.cancellation.first_charge_observed())
            {
                std::thread::yield_now();
            }
        }
        if controlled {
            write_byte(output, TRANSFORM_STARTED)?;
            write_string(output, &request_id)?;
            output.flush()?;
        }
        Ok(())
    }

    fn transform_batch(
        &self,
        members: Vec<BatchMemberCommand>,
        output: &mut impl Write,
    ) -> io::Result<()> {
        let Some(engine) = &self.engine else {
            return write_failure(
                output,
                &worker_failure("FXWB1001", None, "worker has not been initialized"),
            );
        };
        let (outcomes, _) = match collect_batch_outcomes(engine, members) {
            Ok(value) => value,
            Err(failure) => return write_failure(output, &failure),
        };
        write_batch_outcomes(output, outcomes)
    }

    fn incremental_transform_batch(
        &self,
        members: Vec<BatchMemberCommand>,
        protocol_version: Option<u32>,
        output: &mut impl Write,
    ) -> io::Result<()> {
        let Some(engine) = &self.engine else {
            return write_failure(
                output,
                &worker_failure("FXWB1001", None, "worker has not been initialized"),
            );
        };
        incremental_batch::execute(engine, members, protocol_version, output)
    }

    fn begin_active_cancellation_batch(
        &mut self,
        mut members: Vec<BatchMemberCommand>,
        cancel_at_index: usize,
        first_charge_barrier: bool,
        output: &mut impl Write,
    ) -> io::Result<()> {
        if self.active.is_some() {
            return write_failure(
                output,
                &worker_failure("FXWB1003", None, "worker already has an active invocation"),
            );
        }
        let Some(engine) = &self.engine else {
            return write_failure(
                output,
                &worker_failure("FXWB1001", None, "worker has not been initialized"),
            );
        };
        let remaining = members.split_off(cancel_at_index + 1);
        let target = members.pop().expect("validated active batch target");
        let (outcomes, response_bytes) = match collect_batch_outcomes(engine, members) {
            Ok(value) => value,
            Err(failure) => return write_failure(output, &failure),
        };
        let cancellation = if first_charge_barrier {
            WorkbenchCancellation::with_first_charge_barrier()
        } else {
            WorkbenchCancellation::new()
        };
        start_transform(
            Arc::clone(engine),
            target.request_id.clone(),
            cancellation.clone(),
            self.events.clone(),
        );
        self.active = Some(ActiveInvocation {
            request_id: target.request_id.clone(),
            cancellation,
            completion: ActiveCompletion::Batch {
                outcomes,
                remaining,
                response_bytes,
            },
        });
        if first_charge_barrier {
            while !self
                .active
                .as_ref()
                .is_some_and(|value| value.cancellation.first_charge_observed())
            {
                std::thread::yield_now();
            }
        }
        write_byte(output, BATCH_MEMBER_STARTED)?;
        write_u32(
            output,
            u32::try_from(cancel_at_index).expect("bounded batch index fits u32"),
        )?;
        write_string(output, &target.request_id)?;
        output.flush()
    }

    fn batch_loss_probe(
        &self,
        request_ids: Vec<String>,
        park_at_index: usize,
        output: &mut impl Write,
    ) -> io::Result<()> {
        let Some(engine) = &self.engine else {
            return write_failure(
                output,
                &worker_failure("FXWB1001", None, "worker has not been initialized"),
            );
        };

        write_byte(output, BATCH_ACKNOWLEDGED)?;
        write_u32(
            output,
            u32::try_from(request_ids.len()).expect("bounded batch count fits u32"),
        )?;
        output.flush()?;

        for (index, request_id) in request_ids.into_iter().enumerate() {
            write_byte(output, BATCH_MEMBER_STARTED)?;
            write_u32(
                output,
                u32::try_from(index).expect("bounded batch index fits u32"),
            )?;
            write_string(output, &request_id)?;
            output.flush()?;

            if index == park_at_index {
                loop {
                    std::thread::park();
                }
            }

            let _ = engine.transform(&request_id);
            write_byte(output, BATCH_MEMBER_FINISHED)?;
            write_u32(
                output,
                u32::try_from(index).expect("bounded batch index fits u32"),
            )?;
            write_string(output, &request_id)?;
            output.flush()?;
        }

        unreachable!("the validated loss-probe park index is always reached")
    }

    fn batch_transfer_loss_probe(
        &self,
        request_ids: Vec<String>,
        truncate_at_index: usize,
        output: &mut impl Write,
    ) -> io::Result<()> {
        let Some(engine) = &self.engine else {
            return write_failure(
                output,
                &worker_failure("FXWB1001", None, "worker has not been initialized"),
            );
        };

        write_byte(output, BATCH_ACKNOWLEDGED)?;
        write_u32(
            output,
            u32::try_from(request_ids.len()).expect("bounded batch count fits u32"),
        )?;
        output.flush()?;

        let mut response_bytes = 1_usize + size_of::<u32>();
        for (index, request_id) in request_ids.into_iter().enumerate() {
            let result = engine.transform(&request_id).map_err(|failure| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "batch transfer-loss probe requires a successful member: {}",
                        failure.code
                    ),
                )
            })?;
            let outcome_bytes = [
                1_usize,
                size_of::<u32>(),
                1,
                encoded_string_bytes(&request_id)?,
                encoded_string_bytes(&result)?,
            ]
            .into_iter()
            .try_fold(0_usize, |total, value| {
                checked_wire_add(total, value, "batch transfer-loss response size")
            })?;
            response_bytes = checked_wire_add(
                response_bytes,
                outcome_bytes,
                "batch transfer-loss response size",
            )?;
            if response_bytes > MAX_BATCH_RESPONSE_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("batch transfer-loss response bytes exceed {MAX_BATCH_RESPONSE_BYTES}"),
                ));
            }

            write_byte(output, BATCH_INCREMENTAL_OUTCOME)?;
            write_u32(
                output,
                u32::try_from(index).expect("bounded batch index fits u32"),
            )?;
            write_byte(output, RESULT)?;
            write_string(output, &request_id)?;
            if index == truncate_at_index {
                if result.len() < 2 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "batch transfer-loss probe requires at least two result bytes",
                    ));
                }
                write_u32(
                    output,
                    u32::try_from(result.len()).expect("bounded result length fits u32"),
                )?;
                output.write_all(&result.as_bytes()[..result.len() / 2])?;
                output.flush()?;
                loop {
                    std::thread::park();
                }
            }
            write_string(output, &result)?;
            output.flush()?;
        }

        unreachable!("the validated transfer-loss probe index is always reached")
    }
}

fn write_unsupported_batch_protocol(version: u32, output: &mut impl Write) -> io::Result<()> {
    write_failure(
        output,
        &worker_failure(
            "FXWB1005",
            None,
            &format!("unsupported incremental batch protocol version: {version}"),
        ),
    )
}

fn write_unknown_operation(operation: u8, output: &mut impl Write) -> io::Result<()> {
    write_failure(
        output,
        &worker_failure(
            "FXWB1002",
            None,
            &format!("unknown worker operation: {operation}"),
        ),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LoopControl {
    Continue,
    Stop,
}

fn start_transform(
    engine: Arc<ExperimentalEngine>,
    request_id: String,
    cancellation: WorkbenchCancellation,
    events: mpsc::SyncSender<Event>,
) {
    std::thread::spawn(move || {
        let result = engine.transform_with_cancellation(&request_id, cancellation);
        let _ = events.send(Event::Completed { request_id, result });
    });
}

fn write_transform_result(
    output: &mut impl Write,
    request_id: &str,
    result: Result<String, WorkbenchFailure>,
) -> io::Result<()> {
    match result {
        Ok(result) => {
            write_byte(output, RESULT)?;
            write_string(output, request_id)?;
            write_string(output, &result)?;
            output.flush()
        }
        Err(failure) => write_failure(output, &failure),
    }
}

fn write_measured_transform_result(
    output: &mut impl Write,
    request_id: &str,
    result: Result<String, WorkbenchFailure>,
    decode_elapsed: Duration,
    queue_elapsed: Duration,
    execution_elapsed: Duration,
) -> io::Result<()> {
    match result {
        Ok(result) => {
            write_byte(output, MEASURED_RESULT)?;
            write_string(output, request_id)?;
            write_string(output, &result)?;
            write_u64(output, duration_nanoseconds(decode_elapsed))?;
            write_u64(output, duration_nanoseconds(queue_elapsed))?;
            write_u64(output, duration_nanoseconds(execution_elapsed))?;
            output.flush()
        }
        Err(failure) => write_failure(output, &failure),
    }
}

fn duration_nanoseconds(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn read_commands(events: &mpsc::SyncSender<Event>) {
    let stdin = io::stdin();
    let mut input = BufReader::new(stdin.lock());
    loop {
        let command = read_command(&mut input);
        match command {
            Ok(Some(command)) => {
                if events.send(Event::Command(command)).is_err() {
                    return;
                }
            }
            Ok(None) => {
                let _ = events.send(Event::InputClosed(Ok(())));
                return;
            }
            Err(error) => {
                let _ = events.send(Event::InputClosed(Err(error)));
                return;
            }
        }
    }
}

fn read_command(input: &mut impl Read) -> io::Result<Option<Command>> {
    let operation = match read_byte(input) {
        Ok(operation) => operation,
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    };
    let decode_started = Instant::now();
    let command = match operation {
        INITIALIZE => Command::Initialize {
            source_id: read_string(input, MAX_IDENTITY_BYTES)?,
            source: read_bytes(input, MAX_RESOURCE_BYTES)?,
            stylesheet_id: read_string(input, MAX_IDENTITY_BYTES)?,
            stylesheet: read_bytes(input, MAX_RESOURCE_BYTES)?,
            stylesheet_resources: WorkbenchStylesheetResources::default(),
        },
        INITIALIZE_WITH_STYLESHEET_DEPENDENCY => {
            let source_id = read_string(input, MAX_IDENTITY_BYTES)?;
            let source = read_bytes(input, MAX_RESOURCE_BYTES)?;
            let stylesheet_id = read_string(input, MAX_IDENTITY_BYTES)?;
            let stylesheet = read_bytes(input, MAX_RESOURCE_BYTES)?;
            let dependency_id = read_string(input, MAX_IDENTITY_BYTES)?;
            let dependency = read_bytes(input, MAX_RESOURCE_BYTES)?;
            let admitted = read_boolean(input, "dependency admission")?;
            let denied = read_boolean(input, "dependency denial")?;
            if !admitted && !dependency.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unadmitted dependency must not carry resource bytes",
                ));
            }
            Command::Initialize {
                source_id,
                source,
                stylesheet_id,
                stylesheet,
                stylesheet_resources: WorkbenchStylesheetResources {
                    dependencies: admitted
                        .then_some(WorkbenchResource {
                            identity: dependency_id.clone(),
                            bytes: dependency,
                        })
                        .into_iter()
                        .collect(),
                    denied_identities: denied.then_some(dependency_id).into_iter().collect(),
                },
            }
        }
        TRANSFORM
        | CANCELLED_TRANSFORM
        | CONTROLLED_TRANSFORM
        | UNPAUSED_CONTROLLED_TRANSFORM
        | INSTRUCTION_LIMITED_TRANSFORM
        | MEASURED_TRANSFORM => {
            let request_id = read_string(input, MAX_IDENTITY_BYTES)?;
            let maximum_xslt_instructions = if operation == INSTRUCTION_LIMITED_TRANSFORM {
                Some(
                    usize::try_from(read_u64(input)?)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
                )
            } else {
                None
            };
            let measurement = (operation == MEASURED_TRANSFORM).then(|| CommandMeasurement {
                decode_elapsed: decode_started.elapsed(),
                decoded_at: Instant::now(),
            });
            Command::Transform(TransformCommand {
                request_id,
                cancelled: operation == CANCELLED_TRANSFORM,
                controlled: matches!(
                    operation,
                    CONTROLLED_TRANSFORM | UNPAUSED_CONTROLLED_TRANSFORM
                ),
                first_charge_barrier: operation == CONTROLLED_TRANSFORM,
                maximum_xslt_instructions,
                measurement,
            })
        }
        CANCEL => Command::Cancel {
            request_id: read_string(input, MAX_IDENTITY_BYTES)?,
        },
        TRANSFORM_BATCH => read_transform_batch(input, false)?,
        INCREMENTAL_TRANSFORM_BATCH => read_incremental_transform_batch(input, false)?,
        CONTROLLED_INCREMENTAL_TRANSFORM_BATCH => read_incremental_transform_batch(input, true)?,
        VERSIONED_INCREMENTAL_TRANSFORM_BATCH => read_versioned_incremental_transform_batch(input)?,
        CONTROLLED_TRANSFORM_BATCH => read_transform_batch(input, true)?,
        BATCH_LOSS_PROBE => read_batch_loss_probe(input)?,
        BATCH_TRANSFER_LOSS_PROBE => read_batch_transfer_loss_probe(input)?,
        ACTIVE_CANCELLATION_BATCH => read_active_cancellation_batch(input)?,
        NATURAL_CANCELLATION_BATCH => read_natural_cancellation_batch(input)?,
        SHUTDOWN => Command::Shutdown,
        NON_COOPERATING_PROBE => Command::NonCooperatingProbe {
            request_id: read_string(input, MAX_IDENTITY_BYTES)?,
        },
        _ => Command::Unknown(operation),
    };
    Ok(Some(command))
}

fn read_batch_loss_probe(input: &mut impl Read) -> io::Result<Command> {
    let Command::TransformBatch { members } = read_transform_batch(input, false)? else {
        unreachable!("uncontrolled batch decoder always returns a transform batch")
    };
    let park_at_index =
        usize::try_from(read_u32(input)?).expect("u32 index always fits supported Rust targets");
    if park_at_index >= members.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "batch loss-probe index {park_at_index} is outside 0..{}",
                members.len()
            ),
        ));
    }
    Ok(Command::BatchLossProbe {
        request_ids: members
            .into_iter()
            .map(|member| member.request_id)
            .collect(),
        park_at_index,
    })
}

fn read_incremental_transform_batch(
    input: &mut impl Read,
    controlled: bool,
) -> io::Result<Command> {
    let Command::TransformBatch { members } = read_transform_batch(input, controlled)? else {
        unreachable!("batch decoder always returns a transform batch")
    };
    Ok(Command::IncrementalTransformBatch {
        members,
        protocol_version: None,
    })
}

fn read_versioned_incremental_transform_batch(input: &mut impl Read) -> io::Result<Command> {
    let version = read_u32(input)?;
    let payload_length = usize::try_from(read_u32(input)?)
        .expect("u32 payload length always fits supported Rust targets");
    let envelope_bytes = 1_usize
        .checked_add(size_of::<u32>() * 2)
        .and_then(|value| value.checked_add(payload_length))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "batch envelope overflow"))?;
    if envelope_bytes > MAX_BATCH_COMMAND_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("batch command bytes {envelope_bytes} exceed {MAX_BATCH_COMMAND_BYTES}"),
        ));
    }

    let mut payload = vec![0_u8; payload_length];
    input.read_exact(&mut payload)?;
    if version != INCREMENTAL_BATCH_PROTOCOL_VERSION {
        return Ok(Command::UnsupportedBatchProtocolVersion(version));
    }

    let mut payload = io::Cursor::new(payload);
    let Command::TransformBatch { members } = read_transform_batch(&mut payload, true)? else {
        unreachable!("controlled batch decoder always returns a transform batch")
    };
    if payload.position() != payload.get_ref().len() as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "incremental batch payload contains trailing bytes",
        ));
    }
    Ok(Command::IncrementalTransformBatch {
        members,
        protocol_version: Some(version),
    })
}

fn read_batch_transfer_loss_probe(input: &mut impl Read) -> io::Result<Command> {
    let Command::TransformBatch { members } = read_transform_batch(input, false)? else {
        unreachable!("uncontrolled batch decoder always returns a transform batch")
    };
    let truncate_at_index =
        usize::try_from(read_u32(input)?).expect("u32 index always fits supported Rust targets");
    if truncate_at_index >= members.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "batch transfer-loss probe index {truncate_at_index} is outside 0..{}",
                members.len()
            ),
        ));
    }
    Ok(Command::BatchTransferLossProbe {
        request_ids: members
            .into_iter()
            .map(|member| member.request_id)
            .collect(),
        truncate_at_index,
    })
}

fn read_active_cancellation_batch(input: &mut impl Read) -> io::Result<Command> {
    let Command::TransformBatch { members } = read_transform_batch(input, false)? else {
        unreachable!("uncontrolled batch decoder always returns a transform batch")
    };
    let cancel_at_index =
        usize::try_from(read_u32(input)?).expect("u32 index always fits supported Rust targets");
    if cancel_at_index >= members.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "active cancellation batch index {cancel_at_index} is outside 0..{}",
                members.len()
            ),
        ));
    }
    Ok(Command::ActiveCancellationBatch {
        members,
        cancel_at_index,
        first_charge_barrier: true,
    })
}

fn read_natural_cancellation_batch(input: &mut impl Read) -> io::Result<Command> {
    let Command::ActiveCancellationBatch {
        members,
        cancel_at_index,
        ..
    } = read_active_cancellation_batch(input)?
    else {
        unreachable!("active batch decoder always returns an active batch")
    };
    Ok(Command::ActiveCancellationBatch {
        members,
        cancel_at_index,
        first_charge_barrier: false,
    })
}

fn read_transform_batch(input: &mut impl Read, controlled: bool) -> io::Result<Command> {
    let count =
        usize::try_from(read_u32(input)?).expect("u32 count always fits supported Rust targets");
    if count == 0 || count > MAX_BATCH_MEMBERS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("batch member count {count} is outside 1..={MAX_BATCH_MEMBERS}"),
        ));
    }
    let mut members = Vec::with_capacity(count);
    let mut command_bytes = 1_usize + size_of::<u32>();
    for _ in 0..count {
        let request_id = read_string(input, MAX_IDENTITY_BYTES)?;
        command_bytes = checked_wire_add(
            command_bytes,
            encoded_string_bytes(&request_id)?,
            "batch command size",
        )?;
        if command_bytes > MAX_BATCH_COMMAND_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("batch command bytes {command_bytes} exceed {MAX_BATCH_COMMAND_BYTES}"),
            ));
        }
        let (cancelled, maximum_xslt_instructions) = if controlled {
            command_bytes = checked_wire_add(command_bytes, 2, "batch command size")?;
            let cancelled = read_boolean(input, "batch member cancellation")?;
            let has_instruction_limit = read_boolean(input, "batch member instruction limit")?;
            if cancelled && has_instruction_limit {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "batch member cannot combine pre-cancellation with an instruction limit",
                ));
            }
            let maximum = if has_instruction_limit {
                command_bytes =
                    checked_wire_add(command_bytes, size_of::<u64>(), "batch command size")?;
                Some(
                    usize::try_from(read_u64(input)?)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
                )
            } else {
                None
            };
            (cancelled, maximum)
        } else {
            (false, None)
        };
        if command_bytes > MAX_BATCH_COMMAND_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("batch command bytes {command_bytes} exceed {MAX_BATCH_COMMAND_BYTES}"),
            ));
        }
        members.push(BatchMemberCommand {
            request_id,
            cancelled,
            maximum_xslt_instructions,
        });
    }
    Ok(Command::TransformBatch { members })
}

fn worker_failure(code: &str, request_id: Option<String>, detail: &str) -> WorkbenchFailure {
    WorkbenchFailure {
        code: code.to_owned(),
        category: "invalid".to_owned(),
        request_id,
        location: None,
        detail: detail.to_owned(),
    }
}

enum Command {
    Initialize {
        source_id: String,
        source: Vec<u8>,
        stylesheet_id: String,
        stylesheet: Vec<u8>,
        stylesheet_resources: WorkbenchStylesheetResources,
    },
    Transform(TransformCommand),
    Cancel {
        request_id: String,
    },
    TransformBatch {
        members: Vec<BatchMemberCommand>,
    },
    IncrementalTransformBatch {
        members: Vec<BatchMemberCommand>,
        protocol_version: Option<u32>,
    },
    UnsupportedBatchProtocolVersion(u32),
    BatchLossProbe {
        request_ids: Vec<String>,
        park_at_index: usize,
    },
    BatchTransferLossProbe {
        request_ids: Vec<String>,
        truncate_at_index: usize,
    },
    ActiveCancellationBatch {
        members: Vec<BatchMemberCommand>,
        cancel_at_index: usize,
        first_charge_barrier: bool,
    },
    Shutdown,
    NonCooperatingProbe {
        request_id: String,
    },
    Unknown(u8),
}

struct TransformCommand {
    request_id: String,
    cancelled: bool,
    controlled: bool,
    first_charge_barrier: bool,
    maximum_xslt_instructions: Option<usize>,
    measurement: Option<CommandMeasurement>,
}

struct BatchMemberCommand {
    request_id: String,
    cancelled: bool,
    maximum_xslt_instructions: Option<usize>,
}

struct CommandMeasurement {
    decode_elapsed: Duration,
    decoded_at: Instant,
}

enum Event {
    Command(Command),
    Completed {
        request_id: String,
        result: Result<String, WorkbenchFailure>,
    },
    InputClosed(io::Result<()>),
}

struct ActiveInvocation {
    request_id: String,
    cancellation: WorkbenchCancellation,
    completion: ActiveCompletion,
}

enum ActiveCompletion {
    Single,
    Batch {
        outcomes: Vec<BatchOutcome>,
        remaining: Vec<BatchMemberCommand>,
        response_bytes: usize,
    },
}

fn read_byte(input: &mut impl Read) -> io::Result<u8> {
    let mut value = [0_u8; 1];
    input.read_exact(&mut value)?;
    Ok(value[0])
}

fn read_boolean(input: &mut impl Read, field: &str) -> io::Result<bool> {
    match read_byte(input)? {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{field} flag must be zero or one, received {value}"),
        )),
    }
}

fn read_u64(input: &mut impl Read) -> io::Result<u64> {
    let mut value = [0_u8; 8];
    input.read_exact(&mut value)?;
    Ok(u64::from_le_bytes(value))
}

fn read_u32(input: &mut impl Read) -> io::Result<u32> {
    let mut value = [0_u8; 4];
    input.read_exact(&mut value)?;
    Ok(u32::from_le_bytes(value))
}

fn write_u64(output: &mut impl Write, value: u64) -> io::Result<()> {
    output.write_all(&value.to_le_bytes())
}

fn write_u32(output: &mut impl Write, value: u32) -> io::Result<()> {
    output.write_all(&value.to_le_bytes())
}

fn read_bytes(input: &mut impl Read, maximum: usize) -> io::Result<Vec<u8>> {
    let mut length = [0_u8; 4];
    input.read_exact(&mut length)?;
    let length = usize::try_from(u32::from_le_bytes(length))
        .expect("u32 length always fits supported Rust targets");
    if length > maximum {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("frame length {length} exceeds {maximum}"),
        ));
    }
    let mut bytes = vec![0_u8; length];
    input.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn read_string(input: &mut impl Read, maximum: usize) -> io::Result<String> {
    String::from_utf8(read_bytes(input, maximum)?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn write_byte(output: &mut impl Write, value: u8) -> io::Result<()> {
    output.write_all(&[value])
}

fn write_string(output: &mut impl Write, value: &str) -> io::Result<()> {
    let length = u32::try_from(value.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "response is too large"))?;
    output.write_all(&length.to_le_bytes())?;
    output.write_all(value.as_bytes())
}

fn encoded_string_bytes(value: &str) -> io::Result<usize> {
    checked_wire_add(size_of::<u32>(), value.len(), "encoded string size")
}

fn encoded_failure_bytes(failure: &WorkbenchFailure) -> io::Result<usize> {
    let location = failure.location.as_ref();
    let start = location
        .map(|value| value.start.to_string())
        .unwrap_or_default();
    let end = location
        .map(|value| value.end.to_string())
        .unwrap_or_default();
    let fields = [
        failure.code.as_str(),
        failure.category.as_str(),
        failure.request_id.as_deref().unwrap_or_default(),
        location.map_or("", |value| value.resource.as_str()),
        start.as_str(),
        end.as_str(),
        failure.detail.as_str(),
    ];
    fields.into_iter().try_fold(0_usize, |bytes, field| {
        checked_wire_add(bytes, encoded_string_bytes(field)?, "encoded failure size")
    })
}

fn checked_wire_add(left: usize, right: usize, field: &str) -> io::Result<usize> {
    left.checked_add(right)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("{field} overflow")))
}

fn write_failure(output: &mut impl Write, failure: &WorkbenchFailure) -> io::Result<()> {
    write_byte(output, ERROR)?;
    write_failure_fields(output, failure)?;
    output.flush()
}

fn write_failure_fields(output: &mut impl Write, failure: &WorkbenchFailure) -> io::Result<()> {
    write_string(output, &failure.code)?;
    write_string(output, &failure.category)?;
    write_string(output, failure.request_id.as_deref().unwrap_or_default())?;
    write_string(
        output,
        failure
            .location
            .as_ref()
            .map_or("", |location| &location.resource),
    )?;
    write_string(
        output,
        &failure
            .location
            .as_ref()
            .map(|location| location.start.to_string())
            .unwrap_or_default(),
    )?;
    write_string(
        output,
        &failure
            .location
            .as_ref()
            .map(|location| location.end.to_string())
            .unwrap_or_default(),
    )?;
    write_string(output, &failure.detail)
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, sync::mpsc};

    use super::{
        ACTIVE_CANCELLATION_BATCH, BATCH_ACKNOWLEDGED, BATCH_INCREMENTAL_END,
        BATCH_INCREMENTAL_OUTCOME, BATCH_LOSS_PROBE, BATCH_RESULT, BATCH_TRANSFER_LOSS_PROBE,
        CONTROLLED_INCREMENTAL_TRANSFORM_BATCH, CONTROLLED_TRANSFORM_BATCH, Command, ERROR,
        EVENT_QUEUE_CAPACITY, Event, INCREMENTAL_BATCH_PROTOCOL_VERSION,
        INITIALIZE_WITH_STYLESHEET_DEPENDENCY, MAX_BATCH_COMMAND_BYTES, MAX_IDENTITY_BYTES,
        MEASURED_RESULT, MEASURED_TRANSFORM, READY, RESULT, SHUTDOWN, Supervisor, TRANSFORM_BATCH,
        TransformCommand, VERSIONED_BATCH_ACKNOWLEDGED, VERSIONED_INCREMENTAL_TRANSFORM_BATCH,
        read_byte, read_command, read_string, read_u32, read_u64,
    };

    fn push_bytes(frame: &mut Vec<u8>, value: &[u8]) {
        frame.extend_from_slice(
            &u32::try_from(value.len())
                .expect("test frame length")
                .to_le_bytes(),
        );
        frame.extend_from_slice(value);
    }

    fn push_controlled_member(
        frame: &mut Vec<u8>,
        identity: &[u8],
        cancelled: bool,
        maximum: Option<u64>,
    ) {
        push_bytes(frame, identity);
        frame.push(u8::from(cancelled));
        frame.push(u8::from(maximum.is_some()));
        if let Some(maximum) = maximum {
            frame.extend_from_slice(&maximum.to_le_bytes());
        }
    }

    fn versioned_incremental_frame(
        version: u32,
        members: &[(&[u8], bool, Option<u64>)],
    ) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(
            &u32::try_from(members.len())
                .expect("test member count")
                .to_le_bytes(),
        );
        for (identity, cancelled, maximum) in members {
            push_controlled_member(&mut payload, identity, *cancelled, *maximum);
        }

        let mut frame = vec![VERSIONED_INCREMENTAL_TRANSFORM_BATCH];
        frame.extend_from_slice(&version.to_le_bytes());
        frame.extend_from_slice(
            &u32::try_from(payload.len())
                .expect("test payload length")
                .to_le_bytes(),
        );
        frame.extend_from_slice(&payload);
        frame
    }

    fn initialize_frame(dependency: &[u8], admitted: bool, denied: bool) -> Vec<u8> {
        let mut frame = vec![INITIALIZE_WITH_STYLESHEET_DEPENDENCY];
        for value in [
            b"urn:fastxslt:worker-resource-diagnostic:source".as_slice(),
            b"<source/>",
            b"https://example.invalid/styles/main.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="dependency.xsl"/><xsl:variable name="greeting">hello</xsl:variable></xsl:stylesheet>"#,
            b"https://example.invalid/styles/dependency.xsl",
            dependency,
        ] {
            push_bytes(&mut frame, value);
        }
        frame.extend_from_slice(&[u8::from(admitted), u8::from(denied)]);
        frame
    }

    #[test]
    fn worker_failure_envelope_preserves_resource_authority_categories() {
        const STYLESHEET_ID: &str = "https://example.invalid/styles/main.xsl";
        const DEPENDENCY_ID: &str = "https://example.invalid/styles/dependency.xsl";
        for (denied, expected_code, expected_category) in [
            (false, "FXRS0002", "missing-resource"),
            (true, "FXRS0003", "denied"),
        ] {
            let command = read_command(&mut Cursor::new(initialize_frame(&[], false, denied)))
                .expect("read initialization")
                .expect("initialization command");
            let (events, _) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
            let mut supervisor = Supervisor::new(events);
            let mut encoded = Vec::new();
            supervisor
                .handle_command(command, &mut encoded)
                .expect("handle initialization");
            let mut input = Cursor::new(encoded);
            assert_eq!(read_byte(&mut input).expect("error tag"), ERROR);
            let fields = (0..7)
                .map(|_| read_string(&mut input, MAX_IDENTITY_BYTES).expect("failure field"))
                .collect::<Vec<_>>();
            assert_eq!(fields[0], expected_code);
            assert_eq!(fields[1], expected_category);
            assert_eq!(fields[3], STYLESHEET_ID);
            assert!(fields[6].contains(DEPENDENCY_ID));
        }
    }

    #[test]
    fn worker_dependency_initialization_executes_admitted_module() {
        let dependency = br#"<out xsl:version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:value-of select="$greeting"/></out>"#;
        let command = read_command(&mut Cursor::new(initialize_frame(dependency, true, false)))
            .expect("read initialization")
            .expect("initialization command");
        let (events, _) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let mut supervisor = Supervisor::new(events);
        let mut encoded = Vec::new();
        supervisor
            .handle_command(command, &mut encoded)
            .expect("handle initialization");
        assert_eq!(encoded, [READY]);
        assert_eq!(
            supervisor
                .engine
                .as_ref()
                .expect("initialized engine")
                .transform("worker-dependency")
                .expect("transform"),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>hello</out>"
        );
    }

    #[test]
    fn measured_transform_preserves_result_and_reports_worker_durations() {
        let dependency = br#"<out xsl:version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:value-of select="$greeting"/></out>"#;
        let initialize = read_command(&mut Cursor::new(initialize_frame(dependency, true, false)))
            .expect("read initialization")
            .expect("initialization command");
        let (events, _) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let mut supervisor = Supervisor::new(events);
        supervisor
            .handle_command(initialize, &mut Vec::new())
            .expect("initialize engine");

        let mut frame = vec![MEASURED_TRANSFORM];
        push_bytes(&mut frame, b"measured-request");
        let command = read_command(&mut Cursor::new(frame))
            .expect("read measured transform")
            .expect("measured transform command");
        assert!(matches!(
            command,
            Command::Transform(TransformCommand {
                measurement: Some(_),
                ..
            })
        ));

        let mut encoded = Vec::new();
        supervisor
            .handle_command(command, &mut encoded)
            .expect("execute measured transform");
        let mut response = Cursor::new(encoded);
        assert_eq!(
            read_byte(&mut response).expect("result tag"),
            MEASURED_RESULT
        );
        assert_eq!(
            read_string(&mut response, MAX_IDENTITY_BYTES).expect("request identity"),
            "measured-request"
        );
        assert_eq!(
            read_string(&mut response, MAX_IDENTITY_BYTES).expect("result"),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>hello</out>"
        );
        let _decode_nanoseconds = read_u64(&mut response).expect("decode duration");
        let _queue_nanoseconds = read_u64(&mut response).expect("queue duration");
        let _execution_nanoseconds = read_u64(&mut response).expect("execution duration");
        assert_eq!(response.position(), response.get_ref().len() as u64);
    }

    #[test]
    fn bounded_batch_keeps_member_failure_independent() {
        let dependency = br#"<out xsl:version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:value-of select="$greeting"/></out>"#;
        let initialize = read_command(&mut Cursor::new(initialize_frame(dependency, true, false)))
            .expect("read initialization")
            .expect("initialization command");
        let (events, _) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let mut supervisor = Supervisor::new(events);
        supervisor
            .handle_command(initialize, &mut Vec::new())
            .expect("initialize engine");

        let mut frame = vec![TRANSFORM_BATCH];
        frame.extend_from_slice(&3_u32.to_le_bytes());
        push_bytes(&mut frame, b"first");
        push_bytes(&mut frame, b"");
        push_bytes(&mut frame, b"third");
        let command = read_command(&mut Cursor::new(frame))
            .expect("read batch")
            .expect("batch command");
        let mut encoded = Vec::new();
        supervisor
            .handle_command(command, &mut encoded)
            .expect("execute batch");

        let mut response = Cursor::new(encoded);
        assert_eq!(read_byte(&mut response).expect("batch tag"), BATCH_RESULT);
        assert_eq!(read_u32(&mut response).expect("member count"), 3);
        for expected_identity in ["first", "", "third"] {
            let kind = read_byte(&mut response).expect("member kind");
            if expected_identity.is_empty() {
                assert_eq!(kind, ERROR);
                let fields = (0..7)
                    .map(|_| read_string(&mut response, MAX_IDENTITY_BYTES).expect("failure field"))
                    .collect::<Vec<_>>();
                assert_eq!(fields[0], "FXWB0003");
                assert_eq!(fields[2], "");
            } else {
                assert_eq!(kind, RESULT);
                assert_eq!(
                    read_string(&mut response, MAX_IDENTITY_BYTES).expect("identity"),
                    expected_identity
                );
                assert_eq!(
                    read_string(&mut response, MAX_IDENTITY_BYTES).expect("result"),
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>hello</out>"
                );
            }
        }
        assert_eq!(response.position(), response.get_ref().len() as u64);
    }

    #[test]
    fn batch_member_count_is_rejected_before_member_allocation() {
        for count in [0_u32, 129] {
            let mut frame = vec![TRANSFORM_BATCH];
            frame.extend_from_slice(&count.to_le_bytes());
            let Err(error) = read_command(&mut Cursor::new(frame)) else {
                panic!("out-of-range batch count must fail");
            };
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
            assert!(error.to_string().contains("batch member count"));
        }
    }

    #[test]
    fn batch_loss_probe_requires_a_member_index_inside_the_bounded_frame() {
        for (park_at_index, accepted) in [(1_u32, true), (3, false)] {
            let mut frame = vec![BATCH_LOSS_PROBE];
            frame.extend_from_slice(&3_u32.to_le_bytes());
            for identity in [b"first".as_slice(), b"second", b"third"] {
                push_bytes(&mut frame, identity);
            }
            frame.extend_from_slice(&park_at_index.to_le_bytes());
            let command = read_command(&mut Cursor::new(frame));
            if accepted {
                assert!(matches!(
                    command.expect("read loss probe"),
                    Some(Command::BatchLossProbe {
                        park_at_index: 1,
                        ..
                    })
                ));
            } else {
                let Err(error) = command else {
                    panic!("out-of-range loss-probe index must fail");
                };
                assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
                assert!(error.to_string().contains("loss-probe index"));
            }
        }
    }

    #[test]
    fn batch_transfer_loss_probe_requires_an_index_inside_the_bounded_frame() {
        for (truncate_at_index, accepted) in [(2_u32, true), (3, false)] {
            let mut frame = vec![BATCH_TRANSFER_LOSS_PROBE];
            frame.extend_from_slice(&3_u32.to_le_bytes());
            for identity in [b"first".as_slice(), b"second", b"third"] {
                push_bytes(&mut frame, identity);
            }
            frame.extend_from_slice(&truncate_at_index.to_le_bytes());
            let command = read_command(&mut Cursor::new(frame));
            if accepted {
                assert!(matches!(
                    command.expect("read transfer-loss probe"),
                    Some(Command::BatchTransferLossProbe {
                        truncate_at_index: 2,
                        ..
                    })
                ));
            } else {
                let Err(error) = command else {
                    panic!("out-of-range transfer-loss probe index must fail");
                };
                assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
                assert!(error.to_string().contains("transfer-loss probe index"));
            }
        }
    }

    #[test]
    fn active_cancellation_batch_requires_a_target_inside_the_bounded_frame() {
        let mut frame = vec![ACTIVE_CANCELLATION_BATCH];
        frame.extend_from_slice(&3_u32.to_le_bytes());
        for identity in [b"first".as_slice(), b"second", b"third"] {
            push_bytes(&mut frame, identity);
        }
        frame.extend_from_slice(&1_u32.to_le_bytes());
        assert!(matches!(
            read_command(&mut Cursor::new(frame)).expect("read active cancellation batch"),
            Some(Command::ActiveCancellationBatch {
                cancel_at_index: 1,
                ..
            })
        ));
    }

    #[test]
    fn controlled_batch_keeps_cancellation_and_budget_failure_independent() {
        let dependency = br#"<out xsl:version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:value-of select="$greeting"/></out>"#;
        let initialize = read_command(&mut Cursor::new(initialize_frame(dependency, true, false)))
            .expect("read initialization")
            .expect("initialization command");
        let (events, _) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let mut supervisor = Supervisor::new(events);
        supervisor
            .handle_command(initialize, &mut Vec::new())
            .expect("initialize engine");

        let mut frame = vec![CONTROLLED_TRANSFORM_BATCH];
        frame.extend_from_slice(&4_u32.to_le_bytes());
        push_controlled_member(&mut frame, b"before", false, None);
        push_controlled_member(&mut frame, b"cancelled", true, None);
        push_controlled_member(&mut frame, b"limited", false, Some(0));
        push_controlled_member(&mut frame, b"after", false, None);
        let command = read_command(&mut Cursor::new(frame))
            .expect("read controlled batch")
            .expect("controlled batch command");
        let mut encoded = Vec::new();
        supervisor
            .handle_command(command, &mut encoded)
            .expect("execute controlled batch");

        let mut response = Cursor::new(encoded);
        assert_eq!(read_byte(&mut response).expect("batch tag"), BATCH_RESULT);
        assert_eq!(read_u32(&mut response).expect("member count"), 4);
        for (identity, expected_failure) in [
            ("before", None),
            ("cancelled", Some(("FXCT0001", "cancelled"))),
            ("limited", Some(("FXCT0002", "limit"))),
            ("after", None),
        ] {
            let kind = read_byte(&mut response).expect("member kind");
            if let Some((code, category)) = expected_failure {
                assert_eq!(kind, ERROR);
                let fields = (0..7)
                    .map(|_| read_string(&mut response, MAX_IDENTITY_BYTES).expect("failure field"))
                    .collect::<Vec<_>>();
                assert_eq!(fields[0], code);
                assert_eq!(fields[1], category);
                assert_eq!(fields[2], identity);
            } else {
                assert_eq!(kind, RESULT);
                assert_eq!(
                    read_string(&mut response, MAX_IDENTITY_BYTES).expect("identity"),
                    identity
                );
                assert_eq!(
                    read_string(&mut response, MAX_IDENTITY_BYTES).expect("result"),
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>hello</out>"
                );
            }
        }
    }

    #[test]
    fn incremental_batch_preserves_controlled_member_outcomes_in_input_order() {
        let dependency = br#"<out xsl:version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:value-of select="$greeting"/></out>"#;
        let initialize = read_command(&mut Cursor::new(initialize_frame(dependency, true, false)))
            .expect("read initialization")
            .expect("initialization command");
        let (events, _) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let mut supervisor = Supervisor::new(events);
        supervisor
            .handle_command(initialize, &mut Vec::new())
            .expect("initialize engine");

        let mut frame = vec![CONTROLLED_INCREMENTAL_TRANSFORM_BATCH];
        frame.extend_from_slice(&4_u32.to_le_bytes());
        push_controlled_member(&mut frame, b"before", false, None);
        push_controlled_member(&mut frame, b"cancelled", true, None);
        push_controlled_member(&mut frame, b"limited", false, Some(0));
        push_controlled_member(&mut frame, b"after", false, None);
        let command = read_command(&mut Cursor::new(frame))
            .expect("read incremental batch")
            .expect("incremental batch command");
        let mut encoded = Vec::new();
        supervisor
            .handle_command(command, &mut encoded)
            .expect("execute incremental batch");

        let mut response = Cursor::new(encoded);
        assert_eq!(
            read_byte(&mut response).expect("acknowledgement"),
            BATCH_ACKNOWLEDGED
        );
        assert_eq!(read_u32(&mut response).expect("member count"), 4);
        for (index, (identity, expected_failure)) in [
            ("before", None),
            ("cancelled", Some(("FXCT0001", "cancelled"))),
            ("limited", Some(("FXCT0002", "limit"))),
            ("after", None),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                read_byte(&mut response).expect("incremental tag"),
                BATCH_INCREMENTAL_OUTCOME
            );
            assert_eq!(
                read_u32(&mut response).expect("member index"),
                u32::try_from(index).expect("test index")
            );
            let kind = read_byte(&mut response).expect("member kind");
            if let Some((code, category)) = expected_failure {
                assert_eq!(kind, ERROR);
                let fields = (0..7)
                    .map(|_| read_string(&mut response, MAX_IDENTITY_BYTES).expect("failure field"))
                    .collect::<Vec<_>>();
                assert_eq!(fields[0], code);
                assert_eq!(fields[1], category);
                assert_eq!(fields[2], identity);
            } else {
                assert_eq!(kind, RESULT);
                assert_eq!(
                    read_string(&mut response, MAX_IDENTITY_BYTES).expect("identity"),
                    identity
                );
                assert_eq!(
                    read_string(&mut response, MAX_IDENTITY_BYTES).expect("result"),
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>hello</out>"
                );
            }
        }
        assert_eq!(
            read_byte(&mut response).expect("incremental end"),
            BATCH_INCREMENTAL_END
        );
        assert_eq!(read_u32(&mut response).expect("completed count"), 4);
        assert_eq!(response.position(), response.get_ref().len() as u64);
    }

    #[test]
    fn versioned_incremental_batch_acknowledges_the_accepted_protocol() {
        let dependency = br#"<out xsl:version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:value-of select="$greeting"/></out>"#;
        let initialize = read_command(&mut Cursor::new(initialize_frame(dependency, true, false)))
            .expect("read initialization")
            .expect("initialization command");
        let (events, _) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let mut supervisor = Supervisor::new(events);
        supervisor
            .handle_command(initialize, &mut Vec::new())
            .expect("initialize engine");

        let frame = versioned_incremental_frame(
            INCREMENTAL_BATCH_PROTOCOL_VERSION,
            &[(b"versioned", false, None)],
        );
        let command = read_command(&mut Cursor::new(frame))
            .expect("read versioned batch")
            .expect("versioned batch command");
        assert!(matches!(
            &command,
            Command::IncrementalTransformBatch {
                protocol_version: Some(INCREMENTAL_BATCH_PROTOCOL_VERSION),
                ..
            }
        ));

        let mut encoded = Vec::new();
        supervisor
            .handle_command(command, &mut encoded)
            .expect("execute versioned batch");
        let mut response = Cursor::new(encoded);
        assert_eq!(
            read_byte(&mut response).expect("versioned acknowledgement"),
            VERSIONED_BATCH_ACKNOWLEDGED
        );
        assert_eq!(
            read_u32(&mut response).expect("protocol version"),
            INCREMENTAL_BATCH_PROTOCOL_VERSION
        );
        assert_eq!(read_u32(&mut response).expect("member count"), 1);
    }

    #[test]
    fn unknown_batch_protocol_is_rejected_before_payload_admission() {
        let unsupported = INCREMENTAL_BATCH_PROTOCOL_VERSION + 1;
        let mut frame = vec![VERSIONED_INCREMENTAL_TRANSFORM_BATCH];
        frame.extend_from_slice(&unsupported.to_le_bytes());
        frame.extend_from_slice(&3_u32.to_le_bytes());
        frame.extend_from_slice(&[0xff, 0xff, 0xff]);
        frame.push(SHUTDOWN);
        let mut input = Cursor::new(frame);

        let command = read_command(&mut input)
            .expect("read unsupported version")
            .expect("unsupported command");
        assert!(matches!(
            command,
            Command::UnsupportedBatchProtocolVersion(version) if version == unsupported
        ));
        assert!(matches!(
            read_command(&mut input).expect("read following command"),
            Some(Command::Shutdown)
        ));

        let (events, _) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let mut supervisor = Supervisor::new(events);
        let mut response = Vec::new();
        supervisor
            .handle_command(
                Command::UnsupportedBatchProtocolVersion(unsupported),
                &mut response,
            )
            .expect("report unsupported version");
        let mut response = Cursor::new(response);
        assert_eq!(read_byte(&mut response).expect("failure tag"), ERROR);
        let fields = (0..7)
            .map(|_| read_string(&mut response, MAX_IDENTITY_BYTES).expect("failure field"))
            .collect::<Vec<_>>();
        assert_eq!(fields[0], "FXWB1005");
        assert!(fields[6].contains(&unsupported.to_string()));
    }

    #[test]
    fn versioned_batch_rejects_oversized_envelope_before_reading_payload() {
        let mut frame = vec![VERSIONED_INCREMENTAL_TRANSFORM_BATCH];
        frame.extend_from_slice(&INCREMENTAL_BATCH_PROTOCOL_VERSION.to_le_bytes());
        frame.extend_from_slice(
            &u32::try_from(MAX_BATCH_COMMAND_BYTES)
                .expect("test command ceiling fits u32")
                .to_le_bytes(),
        );
        let Err(error) = read_command(&mut Cursor::new(frame)) else {
            panic!("oversized versioned envelope must fail");
        };
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("batch command bytes"));
    }

    #[test]
    fn versioned_batch_rejects_trailing_payload_bytes() {
        let mut frame = versioned_incremental_frame(
            INCREMENTAL_BATCH_PROTOCOL_VERSION,
            &[(b"versioned", false, None)],
        );
        let payload_length = u32::from_le_bytes(
            frame[5..9]
                .try_into()
                .expect("versioned payload length field"),
        );
        frame[5..9].copy_from_slice(&(payload_length + 1).to_le_bytes());
        frame.push(0xff);

        let Err(error) = read_command(&mut Cursor::new(frame)) else {
            panic!("trailing versioned payload bytes must fail");
        };
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("trailing bytes"));
    }

    #[test]
    fn worker_dependency_initialization_rejects_invalid_framing() {
        let mut invalid_flag = initialize_frame(&[], false, false);
        *invalid_flag.last_mut().expect("denial flag") = 2;
        let Err(invalid_flag) = read_command(&mut Cursor::new(invalid_flag)) else {
            panic!("invalid flag must reject framing");
        };
        assert_eq!(invalid_flag.kind(), std::io::ErrorKind::InvalidData);

        let Err(unadmitted_bytes) =
            read_command(&mut Cursor::new(initialize_frame(b"bytes", false, false)))
        else {
            panic!("unadmitted bytes must reject framing");
        };
        assert_eq!(unadmitted_bytes.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn worker_event_queue_applies_backpressure_after_one_decoded_event() {
        let (events, _incoming) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        assert!(events.try_send(Event::Command(Command::Shutdown)).is_ok());
        assert!(matches!(
            events.try_send(Event::Command(Command::Shutdown)),
            Err(mpsc::TrySendError::Full(Event::Command(Command::Shutdown)))
        ));
    }
}
