//! Length-prefixed isolated transport for the ASP.NET boundary workbench.

use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    sync::{Arc, mpsc},
};

use fastxslt::workbench::{
    ExperimentalEngine, WorkbenchCancellation, WorkbenchFailure, WorkbenchLimits,
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
const READY: u8 = 0x81;
const RESULT: u8 = 0x82;
const STOPPED: u8 = 0x83;
const PROBE_STARTED: u8 = 0x84;
const TRANSFORM_STARTED: u8 = 0x85;
const ERROR: u8 = 0xff;
const MAX_IDENTITY_BYTES: usize = 4_096;
const MAX_RESOURCE_BYTES: usize = 1_048_576;

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());
    let (events, incoming) = mpsc::channel();
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
    events: mpsc::Sender<Event>,
}

impl Supervisor {
    fn new(events: mpsc::Sender<Event>) -> Self {
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
                    self.active = None;
                    write_transform_result(output, &request_id, result)?;
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
            } => match ExperimentalEngine::new(
                source_id,
                source,
                stylesheet_id,
                stylesheet,
                WorkbenchLimits::default(),
            ) {
                Ok(initialized) => {
                    self.engine = Some(Arc::new(initialized));
                    write_byte(output, READY)?;
                    output.flush()?;
                }
                Err(failure) => write_failure(output, &failure)?,
            },
            Command::Transform {
                request_id,
                cancelled,
                controlled,
                first_charge_barrier,
                maximum_xslt_instructions,
            } => self.begin_transform(
                request_id,
                cancelled,
                controlled,
                first_charge_barrier,
                maximum_xslt_instructions,
                output,
            )?,
            Command::Cancel { request_id } => {
                if let Some(invocation) = &self.active
                    && invocation.request_id == request_id
                {
                    invocation.cancellation.cancel();
                }
            }
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
            Command::Unknown(operation) => write_failure(
                output,
                &worker_failure(
                    "FXWB1002",
                    None,
                    &format!("unknown worker operation: {operation}"),
                ),
            )?,
        }
        Ok(LoopControl::Continue)
    }

    fn begin_transform(
        &mut self,
        request_id: String,
        cancelled: bool,
        controlled: bool,
        first_charge_barrier: bool,
        maximum_xslt_instructions: Option<usize>,
        output: &mut impl Write,
    ) -> io::Result<()> {
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
            let result = maximum_xslt_instructions.map_or_else(
                || engine.transform_with_cancellation(&request_id, cancellation),
                |maximum| engine.transform_with_xslt_instruction_limit(&request_id, maximum),
            );
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
    events: mpsc::Sender<Event>,
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

fn read_commands(events: &mpsc::Sender<Event>) {
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
    let command = match operation {
        INITIALIZE => Command::Initialize {
            source_id: read_string(input, MAX_IDENTITY_BYTES)?,
            source: read_bytes(input, MAX_RESOURCE_BYTES)?,
            stylesheet_id: read_string(input, MAX_IDENTITY_BYTES)?,
            stylesheet: read_bytes(input, MAX_RESOURCE_BYTES)?,
        },
        TRANSFORM
        | CANCELLED_TRANSFORM
        | CONTROLLED_TRANSFORM
        | UNPAUSED_CONTROLLED_TRANSFORM
        | INSTRUCTION_LIMITED_TRANSFORM => {
            let request_id = read_string(input, MAX_IDENTITY_BYTES)?;
            let maximum_xslt_instructions = if operation == INSTRUCTION_LIMITED_TRANSFORM {
                Some(
                    usize::try_from(read_u64(input)?)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
                )
            } else {
                None
            };
            Command::Transform {
                request_id,
                cancelled: operation == CANCELLED_TRANSFORM,
                controlled: matches!(
                    operation,
                    CONTROLLED_TRANSFORM | UNPAUSED_CONTROLLED_TRANSFORM
                ),
                first_charge_barrier: operation == CONTROLLED_TRANSFORM,
                maximum_xslt_instructions,
            }
        }
        CANCEL => Command::Cancel {
            request_id: read_string(input, MAX_IDENTITY_BYTES)?,
        },
        SHUTDOWN => Command::Shutdown,
        NON_COOPERATING_PROBE => Command::NonCooperatingProbe {
            request_id: read_string(input, MAX_IDENTITY_BYTES)?,
        },
        _ => Command::Unknown(operation),
    };
    Ok(Some(command))
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
    },
    Transform {
        request_id: String,
        cancelled: bool,
        controlled: bool,
        first_charge_barrier: bool,
        maximum_xslt_instructions: Option<usize>,
    },
    Cancel {
        request_id: String,
    },
    Shutdown,
    NonCooperatingProbe {
        request_id: String,
    },
    Unknown(u8),
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
}

fn read_byte(input: &mut impl Read) -> io::Result<u8> {
    let mut value = [0_u8; 1];
    input.read_exact(&mut value)?;
    Ok(value[0])
}

fn read_u64(input: &mut impl Read) -> io::Result<u64> {
    let mut value = [0_u8; 8];
    input.read_exact(&mut value)?;
    Ok(u64::from_le_bytes(value))
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

fn write_failure(output: &mut impl Write, failure: &WorkbenchFailure) -> io::Result<()> {
    write_byte(output, ERROR)?;
    write_string(output, &failure.code)?;
    write_string(output, &failure.category)?;
    write_string(output, failure.request_id.as_deref().unwrap_or_default())?;
    write_string(output, &failure.detail)?;
    output.flush()
}
