//! Length-prefixed isolated transport for the ASP.NET boundary workbench.

use std::io::{self, BufReader, BufWriter, Read, Write};

use fastxslt::workbench::{ExperimentalEngine, WorkbenchFailure, WorkbenchLimits};

const INITIALIZE: u8 = 1;
const TRANSFORM: u8 = 2;
const SHUTDOWN: u8 = 3;
const NON_COOPERATING_PROBE: u8 = 4;
const READY: u8 = 0x81;
const RESULT: u8 = 0x82;
const STOPPED: u8 = 0x83;
const PROBE_STARTED: u8 = 0x84;
const ERROR: u8 = 0xff;
const MAX_IDENTITY_BYTES: usize = 4_096;
const MAX_RESOURCE_BYTES: usize = 1_048_576;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = BufReader::new(stdin.lock());
    let mut output = BufWriter::new(stdout.lock());
    let mut engine = None;

    loop {
        let operation = match read_byte(&mut input) {
            Ok(operation) => operation,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(error),
        };
        match operation {
            INITIALIZE => {
                let source_id = read_string(&mut input, MAX_IDENTITY_BYTES)?;
                let source = read_bytes(&mut input, MAX_RESOURCE_BYTES)?;
                let stylesheet_id = read_string(&mut input, MAX_IDENTITY_BYTES)?;
                let stylesheet = read_bytes(&mut input, MAX_RESOURCE_BYTES)?;
                match ExperimentalEngine::new(
                    source_id,
                    source,
                    stylesheet_id,
                    stylesheet,
                    WorkbenchLimits::default(),
                ) {
                    Ok(initialized) => {
                        engine = Some(initialized);
                        write_byte(&mut output, READY)?;
                        output.flush()?;
                    }
                    Err(failure) => write_failure(&mut output, &failure)?,
                }
            }
            TRANSFORM => {
                let request_id = read_string(&mut input, MAX_IDENTITY_BYTES)?;
                let Some(engine) = &engine else {
                    write_failure(
                        &mut output,
                        &WorkbenchFailure {
                            code: "FXWB1001".to_owned(),
                            category: "invalid".to_owned(),
                            request_id: Some(request_id),
                            detail: "worker has not been initialized".to_owned(),
                        },
                    )?;
                    continue;
                };
                match engine.transform(&request_id) {
                    Ok(result) => {
                        write_byte(&mut output, RESULT)?;
                        write_string(&mut output, &request_id)?;
                        write_string(&mut output, &result)?;
                        output.flush()?;
                    }
                    Err(failure) => write_failure(&mut output, &failure)?,
                }
            }
            SHUTDOWN => {
                write_byte(&mut output, STOPPED)?;
                output.flush()?;
                return Ok(());
            }
            NON_COOPERATING_PROBE => {
                let request_id = read_string(&mut input, MAX_IDENTITY_BYTES)?;
                write_byte(&mut output, PROBE_STARTED)?;
                write_string(&mut output, &request_id)?;
                output.flush()?;
                loop {
                    std::thread::park();
                }
            }
            _ => {
                write_failure(
                    &mut output,
                    &WorkbenchFailure {
                        code: "FXWB1002".to_owned(),
                        category: "invalid".to_owned(),
                        request_id: None,
                        detail: format!("unknown worker operation: {operation}"),
                    },
                )?;
            }
        }
    }
}

fn read_byte(input: &mut impl Read) -> io::Result<u8> {
    let mut value = [0_u8; 1];
    input.read_exact(&mut value)?;
    Ok(value[0])
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
