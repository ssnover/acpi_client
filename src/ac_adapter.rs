use std::error::Error;
use std::fs::read_dir;
use std::path;

use crate::utils::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Online,
    Offline,
}

pub struct ACAdapterInfo {
    pub name: String,
    pub status: Status,
}

pub fn get_ac_adapter_info(path: &path::Path) -> Result<Vec<ACAdapterInfo>, Box<dyn Error>> {
    let mut results: Vec<ACAdapterInfo> = vec![];

    for entry in read_dir(&path)? {
        let path = entry?.path();
        if !determine_is_battery(parse_entry_file(&path.join("type"))?.unwrap()) {
            let adapter = ACAdapterInfo::new(&path);
            if adapter.is_ok() {
                results.push(adapter?);
            }
        }
    }

    Ok(results)
}

impl ACAdapterInfo {
    pub fn new(path: &path::Path) -> Result<ACAdapterInfo, Box<dyn Error>> {
        let name = String::from(path.file_name().unwrap().to_str().unwrap());
        let status = parse_entry_file(&path.join("online"))?
            .unwrap()
            .trim()
            .to_lowercase();
        let status = if status == "1" {
            Status::Online
        } else if status == "0" {
            Status::Offline
        } else {
            return Err(Box::new(AcpiError(
                format!("Invalid contents in {}", &name).into(),
            )));
        };

        Ok(ACAdapterInfo {
            name: name,
            status: status,
        })
    }
}
