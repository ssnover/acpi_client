use std::error::Error;
use std::fmt;
use std::fs;
use std::io::prelude::*;

pub enum ChargingState {
    Charging,
    Discharging,
}

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

pub fn get_power_supply_info() -> Result<Vec<PowerSupplyInfo>, Box<dyn Error>> {
    let mut results: Vec<PowerSupplyInfo> = vec![];
    let power_supply_path = "/sys/class/power_supply";

    for entry in fs::read_dir(&power_supply_path)? {
        results.push(PowerSupplyInfo::new(&entry?)?);
    }

    Ok(results)
}

impl PowerSupplyInfo {
    pub fn new(entry: &fs::DirEntry) -> Result<PowerSupplyInfo, Box<dyn Error>> {
        let name = entry.file_name().into_string().unwrap();

        let voltage = parse_file_to_u32(entry, "voltage_now", 1000)?;
        let remaining_capacity = parse_file_to_u32(entry, "charge_now", 1000)?;
        let remaining_energy = parse_file_to_u32(entry, "energy_now", 1000)?;
        let present_rate = parse_file_to_u32(entry, "current_now", 1000)?;
        let design_capacity = parse_file_to_u32(entry, "charge_full_design", 1000)?;
        let design_capacity_unit = parse_file_to_u32(entry, "energy_full_design", 1000)?;
        let last_capacity = parse_file_to_u32(entry, "charge_full", 1000)?;
        let last_capacity_unit = parse_file_to_u32(entry, "energy_full", 1000)?;
        let is_battery = match parse_entry_file(entry, "type")? {
            Some(val) => val.to_lowercase() == "battery",
            None => false,
        };
        let state = match parse_entry_file(entry, "status")? {
            Some(val) => {
                if val.trim().to_lowercase() == "charging" {
                    Some(ChargingState::Charging)
                } else if val.trim().to_lowercase() == "discharging" {
                    Some(ChargingState::Discharging)
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
            3600 * remaining_capacity.unwrap() / present_rate.unwrap()
        } else {
            0
        };
        let hours = seconds / 3600;
        seconds = seconds - (3600 * hours);
        let minutes = seconds / 60;
        seconds = seconds - (60 * minutes);

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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_battery {
            let state = match &self.state {
                Some(val) => match val {
                    ChargingState::Charging => "Charging",
                    ChargingState::Discharging => "Discharging",
                },
                None => "",
            };
            write!(
                f,
                "{}: {}, {}%, {:02}:{:02}:{:02} remaining",
                self.name,
                state,
                self.percentage.unwrap(),
                self.hours,
                self.minutes,
                self.seconds
            )
        } else {
            write!(f, "{}", self.name)
        }
    }
}

fn parse_entry_file(
    entry: &fs::DirEntry,
    file: &'static str,
) -> Result<Option<String>, Box<dyn Error>> {
    let mut path = entry.path();
    path.push(file);
    let mut result = String::new();

    if path.is_file() {
        if let Some(filename) = path.to_str() {
            let mut f = fs::File::open(filename)?;
            f.read_to_string(&mut result)?;
            let contents = result.trim();
            return Ok(Some(String::from(contents)));
        }
    }

    Ok(None)
}

fn parse_file_to_u32(
    entry: &fs::DirEntry,
    file: &'static str,
    scalar: u32,
) -> Result<Option<u32>, Box<dyn Error>> {
    let result = match parse_entry_file(entry, file)? {
        Some(val) => Some(val.parse::<u32>()? / scalar),
        None => None,
    };
    Ok(result)
}
