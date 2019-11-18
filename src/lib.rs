use std::error::Error;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::path;

/// Different possible battery charging states.
#[derive(Clone, Copy)]
pub enum ChargingState {
    Charging,
    Discharging,
    Full,
}

/// Metadata pertaining to a power supply including batteries and AC adapters.
pub struct PowerSupplyInfo {
    pub name: String,
    pub remaining_capacity: Option<u32>,
    pub remaining_energy: Option<u32>,
    pub present_rate: Option<u32>,
    pub voltage: Option<u32>,
    pub design_capacity: Option<u32>,
    pub design_capacity_unit: Option<u32>,
    pub last_capacity: Option<u32>,
    pub last_capacity_unit: Option<u32>,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub percentage: Option<u32>,
    pub is_battery: bool,
    pub state: Option<ChargingState>,
}

/// Returns a vector of data on power supplies in the system or any errors encountered.
pub fn get_power_supply_info() -> Result<Vec<PowerSupplyInfo>, Box<dyn Error>> {
    let mut results: Vec<PowerSupplyInfo> = vec![];
    let power_supply_path = path::Path::new("/sys/class/power_supply");

    for entry in fs::read_dir(&power_supply_path)? {
        let path = entry?.path();
        results.push(PowerSupplyInfo::new(&path)?);
    }

    Ok(results)
}

impl PowerSupplyInfo {
    /// Returns a power supply corresponding to a given ACPI device path.
    ///
    /// # Arguments
    ///
    /// * `path` - A path to a directory entry of an ACPI device containing data files for the device
    ///
    /// # Example
    /// ```
    /// use std::path;
    /// let directory = path::Path::new("/sys/class/power_supply/BAT1");
    /// let ps_info = acpi_client::PowerSupplyInfo::new(&directory);
    /// ```
    pub fn new(path: &path::Path) -> Result<PowerSupplyInfo, Box<dyn Error>> {
        let voltage = parse_file_to_u32(&path.join("voltage_now"), 1000)?;
        let remaining_capacity = parse_file_to_u32(&path.join("charge_now"), 1000)?;
        let remaining_energy = parse_file_to_u32(&path.join("energy_now"), 1000)?;
        let present_rate = parse_file_to_u32(&path.join("current_now"), 1000)?;
        let design_capacity = parse_file_to_u32(&path.join("charge_full_design"), 1000)?;
        let design_capacity_unit = parse_file_to_u32(&path.join("energy_full_design"), 1000)?;
        let last_capacity = parse_file_to_u32(&path.join("charge_full"), 1000)?;
        let last_capacity_unit = parse_file_to_u32(&path.join("energy_full"), 1000)?;
        let is_battery = match parse_entry_file(&path.join("type"))? {
            Some(val) => val.to_lowercase() == "battery",
            None => false,
        };
        let state = match parse_entry_file(&path.join("status"))? {
            Some(val) => {
                if val.trim().to_lowercase() == "charging" {
                    Some(ChargingState::Charging)
                } else if val.trim().to_lowercase() == "discharging" {
                    Some(ChargingState::Discharging)
                } else if val.trim().to_lowercase() == "full" {
                    Some(ChargingState::Full)
                } else {
                    None
                }
            }
            None => None,
        };
        let percentage = if remaining_capacity.is_some() && last_capacity.is_some() {
            Some(remaining_capacity.unwrap() * 100 / last_capacity.unwrap())
        } else {
            None
        };
        let mut seconds = if remaining_capacity.is_some() && present_rate.is_some() {
            match state.unwrap() {
                ChargingState::Discharging => {
                    3600 * remaining_capacity.unwrap() / (present_rate.unwrap() + 1)
                }
                ChargingState::Charging => {
                    3600 * (last_capacity.unwrap() - remaining_capacity.unwrap())
                        / present_rate.unwrap()
                }
                _ => 0,
            }
        } else {
            0
        };
        let hours = seconds / 3600;
        seconds = seconds - (3600 * hours);
        let minutes = seconds / 60;
        seconds = seconds - (60 * minutes);

        let name = String::from(path.file_name().unwrap().to_str().unwrap());
        Ok(PowerSupplyInfo {
            name,
            remaining_capacity,
            remaining_energy,
            present_rate,
            voltage,
            design_capacity,
            design_capacity_unit,
            last_capacity,
            last_capacity_unit,
            hours,
            minutes,
            seconds,
            percentage,
            is_battery,
            state,
        })
    }
}

impl fmt::Display for PowerSupplyInfo {
    /// Creates a string representation of a power supply's state from its data.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_battery {
            let state = match &self.state {
                Some(val) => match val {
                    ChargingState::Charging => "Charging",
                    ChargingState::Discharging => "Discharging",
                    ChargingState::Full => "Full",
                },
                None => "",
            };
            let not_full_string = format!(
                ", {:02}:{:02}:{:02}",
                self.hours, self.minutes, self.seconds
            );
            let charge_time_string = match &self.state.unwrap() {
                ChargingState::Charging => format!("{} {}", not_full_string, "until charged"),
                ChargingState::Discharging => format!("{} {}", not_full_string, "remaining"),
                _ => String::from(""),
            };
            write!(
                f,
                "{}: {}, {}%{}",
                self.name,
                state,
                self.percentage.unwrap(),
                charge_time_string
            )
        } else {
            write!(f, "{}", self.name)
        }
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
