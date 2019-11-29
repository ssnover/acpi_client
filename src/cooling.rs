use std::error::Error;
use std::fs::read_dir;
use std::path;

use crate::utils::*;

#[derive(Clone, Copy)]
pub struct CoolingStatus {
    pub current_state: i32,
    pub max_state: i32,
}

pub struct CoolingDevice {
    pub name: String,
    pub state: Option<CoolingStatus>,
    pub device_type: String,
}

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
    pub fn new(path: &path::Path) -> Result<CoolingDevice, Box<dyn Error>> {
        let name = String::from(path.file_name().unwrap().to_str().unwrap());
        let current_state = parse_file_to_i32(&path.join("cur_state"), 1)?.unwrap();
        let max_state = parse_file_to_i32(&path.join("max_state"), 1)?.unwrap();
        let device_type = parse_entry_file(&path.join("type"))?.unwrap();

        let status = if current_state >= 0 {
            Some(CoolingStatus {current_state, max_state})
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
