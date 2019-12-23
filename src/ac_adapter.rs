use std::fs::read_dir;
use std::path;

use crate::utils::*;

/// An enumeration of the states that the AC adapter system can be in.
#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    /// AC adapter is connected and charging.
    Online,
    /// AC adapter is not connected or charging.
    Offline,
}

/// Information about AC adapters plugged into the system.
pub struct ACAdapterInfo {
    /// The name used by ACPI to refer to the adapter.
    pub name: String,
    /// Whether the adapter is plugged in and charging or not.
    pub status: Status,
}

/// Check the ACPI system for all AC adapters the OS knows about.
///
/// # Arguments
///
/// * `path` - The path to AC adapter entries produced by the ACPI subsystem.
pub fn get_ac_adapter_info(path: &path::Path) -> Result<Vec<ACAdapterInfo>, AcpiClientError> {
    let mut results: Vec<ACAdapterInfo> = vec![];

    for entry in read_dir(&path)? {
        let path = entry?.path();
        if !determine_is_battery(parse_entry_file(&path.join("type"))?) {
            let adapter = ACAdapterInfo::new(&path);
            if adapter.is_ok() {
                results.push(adapter?);
            }
        }
    }

    Ok(results)
}

impl ACAdapterInfo {
    /// Create a new AC adapter object from data from the ACPI subsystem.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the ACPI device.
    pub fn new(path: &path::Path) -> Result<ACAdapterInfo, AcpiClientError> {
        let name = get_device_name(path)?;
        let status = parse_entry_file(&path.join("online"))?
            .trim()
            .to_lowercase();
        let status = if status == "1" {
            Status::Online
        } else if status == "0" {
            Status::Offline
        } else {
            return Err(AcpiClientError::InvalidInput(std::io::Error::new(
                std::io::ErrorKind::Other,
                // Safe to unwrap path's string representation at this point as it's done earlier
                format!("Unexpected value in {}", path.to_str().unwrap()),
            )));
        };

        Ok(ACAdapterInfo { name, status })
    }
}
