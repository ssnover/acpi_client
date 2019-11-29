use std::error::Error;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::path;

#[derive(Debug)]
pub struct AcpiError(pub String);

impl fmt::Display for AcpiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "acpi_client error: {}", self.0)
    }
}

impl Error for AcpiError {}

pub fn determine_is_battery(data: String) -> bool {
    data.to_lowercase() == "battery"
}

pub fn is_thermal_sensor(device_path: &path::Path) -> bool {
    let temperature_file_path = device_path.to_path_buf().join("temp");
    temperature_file_path.exists()
}

/// Returns a string parsed from a file in a directory.
///
/// # Arguments
///
/// * `path` - A path to the file to parse
pub fn parse_entry_file(path: &path::Path) -> Result<Option<String>, Box<dyn Error>> {
    let mut result = String::new();

    if path.is_file() {
        let mut f = fs::File::open(path)?;
        f.read_to_string(&mut result)?;
        let result = result.trim();
        return Ok(Some(String::from(result)));
    }

    Ok(None)
}

/// Parses a file and converts the resulting contents to an integer.
///
/// # Arguments
///
/// * `path` - A path to the file to parse
/// * `scalar` - A number to divide the output by before returning it
pub fn parse_file_to_i32(path: &path::Path, scalar: i32) -> Result<Option<i32>, Box<dyn Error>> {
    let result = match parse_entry_file(path)? {
        Some(val) => Some(val.parse::<i32>()? / scalar),
        None => None,
    };
    Ok(result)
}
