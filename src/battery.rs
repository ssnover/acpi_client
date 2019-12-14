use std::error::Error;
use std::fs;
use std::path;
use std::time;

use crate::utils::*;

/// Different possible battery charging states.
#[derive(Clone, Copy)]
pub enum ChargingState {
    Charging,
    Discharging,
    Full,
}

/// Metadata pertaining to a battery.
pub struct BatteryInfo {
    /// The name used by ACPI to refer to the device.
    pub name: String,
    /// The charge remaining in the battery in units of mAh.
    pub remaining_capacity: u32,
    /// The rate at which the charge of the battery is changing in mA.
    pub present_rate: u32,
    /// The current voltage of the battery in mV.
    pub voltage: u32,
    /// The charge available in the battery at the time of manufacture in units of mAh.
    pub design_capacity: u32,
    /// The charge available in the battery at the last time the device was charged to full in
    /// units of mAh.
    pub last_capacity: u32,
    /// The time remaining until the battery reaches full charge or empty.
    pub time_remaining: time::Duration,
    /// The ratio of the remaining charge to the full charge.
    pub percentage: f32,
    /// The state of the battery's charging.
    pub state: ChargingState,
}

/// Returns a vector of data on power supplies in the system or any errors encountered.
///
/// # Arguments
///
/// * `path` - The path to battery entries produced by the ACPI subsystem.
pub fn get_battery_info(path: &path::Path) -> Result<Vec<BatteryInfo>, Box<dyn Error>> {
    let mut results: Vec<BatteryInfo> = vec![];

    for entry in fs::read_dir(&path)? {
        let path = entry?.path();
        if determine_is_battery(parse_entry_file(&path.join("type"))?) {
            let ps = BatteryInfo::new(&path);
            if ps.is_ok() {
                results.push(ps?);
            }
        }
    }

    Ok(results)
}

impl BatteryInfo {
    /// Returns a battery corresponding to a given ACPI device path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the ACPI device.
    ///
    /// # Example
    /// ```
    /// let directory = std::path::Path::new("/sys/class/power_supply/BAT1");
    /// let ps_info = acpi_client::BatteryInfo::new(&directory);
    /// ```
    pub fn new(path: &path::Path) -> Result<BatteryInfo, Box<dyn Error>> {
        // Check whether the system reports energy or capacity
        match determine_reporting_type(&path)? {
            ReportType::Capacity => return parse_capacity_supply(&path),
            ReportType::Energy => return parse_energy_supply(&path),
        }
    }
}

/// Parses a battery ACPI device entry which reports capacity in units of mAh.
///
/// # Arguments
///
/// * `path` - The path to the ACPI device.
fn parse_capacity_supply(path: &path::Path) -> Result<BatteryInfo, Box<dyn Error>> {
    let voltage = parse_file_to_i32(&path.join("voltage_now"), 1000)? as u32;
    let remaining_capacity = parse_file_to_i32(&path.join("charge_now"), 1000)? as u32;
    let present_rate = parse_file_to_i32(&path.join("current_now"), 1000)? as u32;
    let design_capacity =
        parse_file_to_i32(&path.join("charge_full_design"), 1000)? as u32;
    let last_capacity = parse_file_to_i32(&path.join("charge_full"), 1000)? as u32;
    let status_str = parse_entry_file(&path.join("status"))?
        .trim()
        .to_lowercase();
    let state = if status_str == "charging" {
        Some(ChargingState::Charging)
    } else if status_str == "discharging" {
        Some(ChargingState::Discharging)
    } else if status_str == "full" {
        Some(ChargingState::Full)
    } else {
        None
    };
    let percentage = determine_charge_percentage(remaining_capacity, last_capacity);
    let time_remaining = determine_time_to_state_change(
        remaining_capacity,
        last_capacity,
        present_rate,
        state.unwrap(),
    );
    let name = String::from(path.file_name().unwrap().to_str().unwrap());

    Ok(BatteryInfo {
        name,
        remaining_capacity: remaining_capacity,
        present_rate: present_rate,
        voltage: voltage,
        design_capacity: design_capacity,
        last_capacity: last_capacity,
        percentage,
        time_remaining,
        state: state.unwrap(),
    })
}

/// Parses a battery ACPI device entry which reports capacity in units of mWh.
///
/// # Arguments
///
/// * `path` - The path to the ACPI device.
fn parse_energy_supply(path: &path::Path) -> Result<BatteryInfo, Box<dyn Error>> {
    let voltage = parse_file_to_i32(&path.join("voltage_now"), 1000)? as u32;
    let remaining_capacity =
        parse_file_to_i32(&path.join("energy_now"), 1000)? as u32 / voltage;
    let present_rate = parse_file_to_i32(&path.join("current_now"), 1000)? as u32;
    let design_capacity =
        parse_file_to_i32(&path.join("energy_full_design"), 1000)? as u32 / voltage;
    let last_capacity =
        parse_file_to_i32(&path.join("energy_full"), 1000)? as u32 / voltage;
    let status_str = parse_entry_file(&path.join("status"))?
        .trim()
        .to_lowercase();
    let state = if status_str == "charging" {
        Some(ChargingState::Charging)
    } else if status_str == "discharging" {
        Some(ChargingState::Discharging)
    } else if status_str == "full" {
        Some(ChargingState::Full)
    } else {
        None
    };
    let percentage = determine_charge_percentage(remaining_capacity, last_capacity);
    let time_remaining = determine_time_to_state_change(
        remaining_capacity,
        last_capacity,
        present_rate,
        state.unwrap(),
    );
    let name = String::from(path.file_name().unwrap().to_str().unwrap());

    Ok(BatteryInfo {
        name,
        remaining_capacity,
        present_rate,
        voltage,
        design_capacity,
        last_capacity,
        percentage,
        time_remaining,
        state: state.unwrap(),
    })
}

/// Determines the percentage of full charge from the current charge and the full charge
/// measurements.
///
/// # Arguments
///
/// * `remaining_capacity` - The current charge of the battery in mAh.
/// * `full_capacity` - The full charge of the battery in mAh.
fn determine_charge_percentage(remaining_capacity: u32, full_capacity: u32) -> f32 {
    (remaining_capacity as f32) * 100.0 / (full_capacity as f32)
}

/// Determines the amount of time until the battery finishes charging or until the battery is
/// depleted.
///
/// # Arguments
///
/// * `remaining_capacity` - The current charge of the battery in mAh.
/// * `full_capacity` - The full charge of the battery in mAh.
/// * `present_rate` - The rate at which the current charge is changing in mA.
/// * `state` - Whether the battery is charging or discharging energy.
fn determine_time_to_state_change(
    remaining_capacity: u32,
    full_capacity: u32,
    present_rate: u32,
    state: ChargingState,
) -> time::Duration {
    match state {
        ChargingState::Charging => {
            let seconds = (3600 * (full_capacity - remaining_capacity) / (present_rate + 1)) as u64;
            time::Duration::new(seconds, 0)
        }
        ChargingState::Discharging => {
            let seconds = (3600 * remaining_capacity / (present_rate + 1)) as u64;
            time::Duration::new(seconds, 0)
        }
        _ => time::Duration::new(0, 0),
    }
}

/// An enumeration of different types of units with which the ACPI subsystem reports capacity.
#[derive(Clone)]
enum ReportType {
    Capacity,
    Energy,
}

/// Checks the filesystem to determine if the battery reports capacity or energy
///
/// # Arguments
///
/// * `path` - The path to the ACPI device.
fn determine_reporting_type(path: &path::Path) -> Result<ReportType, Box<dyn Error>> {
    let capacity_files = vec!["charge_now", "charge_full", "charge_full_design"];
    let energy_files = vec!["energy_now", "energy_full", "energy_full_design"];
    if capacity_files.iter().all(|file| {
        let mut path_buffer = path::Path::new(path).to_path_buf();
        path_buffer.push(file);
        path_buffer.exists()
    }) {
        return Ok(ReportType::Capacity);
    } else if energy_files.iter().all(|file| {
        let mut path_buffer = path::Path::new(path).to_path_buf();
        path_buffer.push(file);
        path_buffer.exists()
    }) {
        return Ok(ReportType::Energy);
    } else {
        return Err(Box::new(AcpiError(
            "Cannot determine if device supports energy or capacity reporting.".into(),
        )));
    }
}
