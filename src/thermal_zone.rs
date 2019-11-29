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

pub struct TripPoint {
    pub number: u8,
    pub action_type: String,
    pub temperature: f32,
    pub units: Units,
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
        let mut trip_points: Vec<TripPoint> = vec![];
        let current_temperature =
            convert_from_celsius((parse_file_to_i32(&path.join("temp"), 1)?.unwrap() as f32) / 1000., units);

        let mut trip_point_counter: u8 = 0;
        loop {
            if path.join(format!("trip_point_{}_temp", trip_point_counter)).exists() {
               let tp = TripPoint::new(&path, trip_point_counter, units);
               if tp.is_ok() {
                   trip_points.push(tp?);
                   trip_point_counter = trip_point_counter + 1;
               } else {
                   break;
               }
            } else {
                break;
            }
        };

        Ok(ThermalSensor {
            name,
            current_temperature,
            units,
            trip_points,
        })
    }
}

impl TripPoint {
    pub fn new(path: &path::Path, number: u8, units: Units) -> Result<TripPoint, Box<dyn Error>> {
        let action_type = String::from(parse_entry_file(&path.join(format!("trip_point_{}_type", number)))?.unwrap());
        let temperature_c = (parse_file_to_i32(&path.join(format!("trip_point_{}_temp", number)), 1)?.unwrap() as f32) / 1000.; 

        Ok(TripPoint {
            number,
            action_type,
            temperature: convert_from_celsius(temperature_c, units),
            units,
        })
    }

}

fn convert_from_celsius(temperature: f32, units: Units) -> f32 {
    match units {
        Units::Celsius => temperature,
        Units::Fahrenheit => (temperature * 1.8) + 32.,
        Units::Kelvin => temperature + 273.15,
    }
}
