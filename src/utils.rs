use std::fmt;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path;

#[derive(Debug)]
pub enum AcpiClientError {
    Parse(std::num::ParseIntError),
    Io(std::io::Error),
    InvalidInput(std::io::Error),
}

impl fmt::Display for AcpiClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AcpiClientError::Parse(ref err) => write!(f, "Parse error: {}", err),
            AcpiClientError::Io(ref err) => write!(f, "IO error: {}", err),
            AcpiClientError::InvalidInput(ref err) => write!(f, "Invalid input: {}", err),
        }
    }
}

impl std::error::Error for AcpiClientError {
    fn description(&self) -> &str {
        match *self {
            AcpiClientError::Parse(ref err) => std::error::Error::description(err),
            AcpiClientError::Io(ref err) => err.description(),
            AcpiClientError::InvalidInput(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            AcpiClientError::Parse(ref err) => Some(err),
            AcpiClientError::Io(ref err) => Some(err),
            AcpiClientError::InvalidInput(ref err) => Some(err),
        }
    }
}

impl From<std::io::Error> for AcpiClientError {
    fn from(err: std::io::Error) -> AcpiClientError {
        AcpiClientError::Io(err)
    }
}

impl From<std::num::ParseIntError> for AcpiClientError {
    fn from(err: std::num::ParseIntError) -> AcpiClientError {
        AcpiClientError::Parse(err)
    }
}

pub fn determine_is_battery(data: String) -> bool {
    data.to_lowercase() == "battery"
}

pub fn is_thermal_sensor(device_path: &path::Path) -> bool {
    let temperature_file_path = device_path.to_path_buf().join("temp");
    temperature_file_path.exists()
}

pub fn get_device_name(path: &path::Path) -> Result<String, AcpiClientError> {
    let filename = path.file_name().ok_or(AcpiClientError::Io(io::Error::new(
        io::ErrorKind::Other,
        "Path is not a file.",
    )))?;
    let filename_str = filename.to_str().ok_or(AcpiClientError::Io(io::Error::new(
        io::ErrorKind::Other,
        "Filename contains Unicode characters.",
    )))?;
    Ok(String::from(filename_str))
}

/// Returns a string parsed from a file in a directory.
///
/// # Arguments
///
/// * `path` - A path to the file to parse
pub fn parse_entry_file(path: &path::Path) -> Result<String, AcpiClientError> {
    let mut result = String::new();

    if path.is_file() {
        let mut f = fs::File::open(path)?;
        f.read_to_string(&mut result)?;
        let result = result.trim();
        return Ok(String::from(result));
    }

    Err(AcpiClientError::Io(io::Error::new(
        io::ErrorKind::Other,
        "Path is not a file.",
    )))
}

/// Parses a file and converts the resulting contents to an integer.
///
/// # Arguments
///
/// * `path` - A path to the file to parse
/// * `scalar` - A number to divide the output by before returning it
pub fn parse_file_to_i32(path: &path::Path, scalar: i32) -> Result<i32, AcpiClientError> {
    Ok(parse_entry_file(path)?.parse::<i32>()? / scalar)
}
