use std::error::Error;
use std::fs;
use std::path;

use crate::utils::*;

#[derive(Clone, Copy)]
pub enum Units {
    Fahrenheit,
    Celsius,
    Kelvin,
}

pub enum ActionType {
    Passive,
}

pub struct TripPoint {
    pub number: u8,
    pub mode: ActionType,
    pub temperature: f32,
}

pub struct ThermalSensor {
    pub name: String,
    pub current_temperature: f32,
    pub units: Units,
    pub trip_points: Vec<TripPoint>,
}

pub fn get_thermal_sensor_info(
    path: &path::Path,
    units: Units,
) -> Result<Vec<ThermalSensor>, Box<dyn Error>> {
    let mut results: Vec<ThermalSensor> = vec![];

    for entry in fs::read_dir(&path)? {
        let path = entry?.path();
        if is_thermal_sensor(&path) {
            let tz = ThermalSensor::new(&path, units);
            if tz.is_ok() {
                results.push(tz?);
            }
        }
    }

    Ok(results)
}

impl ThermalSensor {
    pub fn new(path: &path::Path, units: Units) -> Result<ThermalSensor, Box<dyn Error>> {
        let name = String::from(path.file_name().unwrap().to_str().unwrap());
        let trip_points: Vec<TripPoint> = vec![];
        let current_temperature =
            (parse_file_to_u32(&path.join("temp"), 1)?.unwrap() as f32) / 1000.;
        let current_temperature = match units {
            Units::Celsius => current_temperature,
            Units::Fahrenheit => (current_temperature * 1.8) + 32.,
            Units::Kelvin => current_temperature + 273.15,
        };

        Ok(ThermalSensor {
            name,
            current_temperature,
            units,
            trip_points,
        })
    }
}
