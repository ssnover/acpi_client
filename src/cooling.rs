use std::error::Error;
use std::fs::read_dir;
use std::path;

use crate::utils::*;

/// State information on a cooling device's activity.
#[derive(Clone, Copy)]
pub struct CoolingStatus {
    /// The current level of the cooling device relative to the max_state
    pub current_state: i32,
    /// The maximum level of activity of the cooling device.
    pub max_state: i32,
}

/// Information about cooling devices available to the system.
pub struct CoolingDevice {
    /// The name used by ACPI to refer to the device.
    pub name: String,
    /// The activity state of the device.
    pub state: Option<CoolingStatus>,
    /// The type of device the cooling device is attached to.
    pub device_type: String,
}

/// Check the ACPI system for all cooling devices available to the system.
///
/// # Arguments
///
/// * `path` - The path to the cooling device entries produced by the ACPI subsystem.
pub fn get_cooling_device_info(path: &path::Path) -> Result<Vec<CoolingDevice>, Box<dyn Error>> {
    let mut results: Vec<CoolingDevice> = vec![];

    for entry in read_dir(&path)? {
        let path = entry?.path();
        if !is_thermal_sensor(&path) {
            let device = CoolingDevice::new(&path);
            if device.is_ok() {
                results.push(device?);
            }
        }
    }

    Ok(results)
}

impl CoolingDevice {
    /// Create a new cooling device object from data from the ACPI subsystem.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the cooling device entry.
    pub fn new(path: &path::Path) -> Result<CoolingDevice, Box<dyn Error>> {
        let name = get_device_name(path)?;
        let current_state = parse_file_to_i32(&path.join("cur_state"), 1)?;
        let max_state = parse_file_to_i32(&path.join("max_state"), 1)?;
        let device_type = parse_entry_file(&path.join("type"))?;

        let status = if current_state >= 0 {
            Some(CoolingStatus {
                current_state,
                max_state,
            })
        } else {
            None
        };
        Ok(CoolingDevice {
            name,
            state: status,
            device_type,
        })
    }
}
