use std::error::Error;
use std::fs;
use std::path;

use crate::utils::*;

/// An enumeration of the units with which the applications is displaying temperature data.
#[derive(Clone, Copy)]
pub enum Units {
    Fahrenheit,
    Celsius,
    Kelvin,
}

/// Information about the temperature at which the system takes action to reduce the temperature of a thermal zone.
pub struct TripPoint {
    /// A numerical identifier for the trip point.
    pub number: u8,
    /// The type of action the system takes when the trip point is reached.
    pub action_type: String,
    /// The temperature marked as a threshold.
    pub temperature: f32,
    /// The units of the temperature data.
    pub units: Units,
}

/// Information about a zone monitored by a temperature sensor.
pub struct ThermalSensor {
    /// The name used by ACPI to refer to the sensor.
    pub name: String,
    /// The current temperature measured by the sensor.
    pub current_temperature: f32,
    /// The units of the temperature data.
    pub units: Units,
    /// A list of the trip points configured for the zone.
    pub trip_points: Vec<TripPoint>,
}

/// Check the ACPI system for all thermal sensors the OS knows about.
///
/// # Arguments
///
/// * `path` - The path to thermal zone entries produced by the ACPI subsystem.
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
    /// Create a new thermal sensor object from data from the ACPI subsystem.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the ACPI device.
    pub fn new(path: &path::Path, units: Units) -> Result<ThermalSensor, Box<dyn Error>> {
        let name = get_device_name(path)?;
        let mut trip_points: Vec<TripPoint> = vec![];
        let current_temperature = convert_from_celsius(
            (parse_file_to_i32(&path.join("temp"), 1)? as f32) / 1000.,
            units,
        );

        let mut trip_point_counter: u8 = 0;
        loop {
            if path
                .join(format!("trip_point_{}_temp", trip_point_counter))
                .exists()
            {
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
        }

        Ok(ThermalSensor {
            name,
            current_temperature,
            units,
            trip_points,
        })
    }
}

impl TripPoint {
    /// Create a new trip point object from data from the ACPI subsystem.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the ACPI device trip points are configured for.
    /// * `number` - The numerical id of the trip point.
    /// * `units` - The units to convert the temperature data to.
    pub fn new(path: &path::Path, number: u8, units: Units) -> Result<TripPoint, Box<dyn Error>> {
        let action_type = String::from(parse_entry_file(
            &path.join(format!("trip_point_{}_type", number)),
        )?);
        let temperature_c =
            (parse_file_to_i32(&path.join(format!("trip_point_{}_temp", number)), 1)? as f32)
                / 1000.;

        Ok(TripPoint {
            number,
            action_type,
            temperature: convert_from_celsius(temperature_c, units),
            units,
        })
    }
}

/// Convert a temperature value to a different scale from degrees Celsius.
///
/// # Arguments
///
/// * `temperature` - The measurement to convert in Celsius.
/// * `units` - The measurement scale to convert to.
fn convert_from_celsius(temperature: f32, units: Units) -> f32 {
    match units {
        Units::Celsius => temperature,
        Units::Fahrenheit => (temperature * 1.8) + 32.,
        Units::Kelvin => temperature + 273.15,
    }
}
