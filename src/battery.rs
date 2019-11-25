use std::error::Error;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::path;
use std::time;

/// Different possible battery charging states.
#[derive(Clone, Copy)]
pub enum ChargingState {
    Charging,
    Discharging,
    Full,
}

/// Metadata pertaining to a battery including batteries and AC adapters.
pub struct BatteryInfo {
    pub name: String,
    pub remaining_capacity: u32,
    pub present_rate: u32,
    pub voltage: u32,
    pub design_capacity: u32,
    pub last_capacity: u32,
    pub time_remaining: time::Duration,
    pub percentage: f32,
    pub state: ChargingState,
}

/// Returns a vector of data on power supplies in the system or any errors encountered.
pub fn get_battery_info() -> Result<Vec<BatteryInfo>, Box<dyn Error>> {
    let mut results: Vec<BatteryInfo> = vec![];
    let power_supply_path = path::Path::new("/sys/class/power_supply");

    for entry in fs::read_dir(&power_supply_path)? {
        let path = entry?.path();
        if determine_is_battery(parse_entry_file(&path.join("type"))?.unwrap()) {
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
    /// * `path` - A path to a directory entry of an ACPI device containing data files for the device
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

fn parse_capacity_supply(path: &path::Path) -> Result<BatteryInfo, Box<dyn Error>> {
    let voltage = parse_file_to_u32(&path.join("voltage_now"), 1000)?.unwrap();
    let remaining_capacity = parse_file_to_u32(&path.join("charge_now"), 1000)?.unwrap();
    let present_rate = parse_file_to_u32(&path.join("current_now"), 1000)?.unwrap();
    let design_capacity = parse_file_to_u32(&path.join("charge_full_design"), 1000)?.unwrap();
    let last_capacity = parse_file_to_u32(&path.join("charge_full"), 1000)?.unwrap();
    let status_str = parse_entry_file(&path.join("status"))?
        .unwrap()
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

fn parse_energy_supply(path: &path::Path) -> Result<BatteryInfo, Box<dyn Error>> {
    let voltage = parse_file_to_u32(&path.join("voltage_now"), 1000)?.unwrap();
    let remaining_capacity = parse_file_to_u32(&path.join("energy_now"), 1000)?.unwrap() / voltage;
    let present_rate = parse_file_to_u32(&path.join("current_now"), 1000)?.unwrap();
    let design_capacity =
        parse_file_to_u32(&path.join("energy_full_design"), 1000)?.unwrap() / voltage;
    let last_capacity = parse_file_to_u32(&path.join("energy_full"), 1000)?.unwrap() / voltage;
    let status_str = parse_entry_file(&path.join("status"))?
        .unwrap()
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

fn determine_is_battery(data: String) -> bool {
    data.to_lowercase() == "battery"
}

fn determine_charge_percentage(remaining_capacity: u32, full_capacity: u32) -> f32 {
    (remaining_capacity as f32) * 100.0 / (full_capacity as f32)
}

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

/// Returns a string parsed from a file in a directory.
///
/// # Arguments
///
/// * `path` - A path to the file to parse
fn parse_entry_file(path: &path::Path) -> Result<Option<String>, Box<dyn Error>> {
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
fn parse_file_to_u32(path: &path::Path, scalar: u32) -> Result<Option<u32>, Box<dyn Error>> {
    let result = match parse_entry_file(path)? {
        Some(val) => Some(val.parse::<u32>()? / scalar),
        None => None,
    };
    Ok(result)
}

#[derive(Clone)]
enum ReportType {
    Capacity,
    Energy,
}

#[derive(Debug)]
struct AcpiError(String);

impl fmt::Display for AcpiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "acpi_client error: {}", self.0)
    }
}

impl Error for AcpiError {}

/// Checks the filesystem to determine if the battery reports capacity or energy
///
/// # Arguments
///
/// * `path` - A path to the device's files
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
