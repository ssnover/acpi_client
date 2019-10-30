use std::fs;
use std::io::Read;

struct PowerSupplyInfo {
    remaining_capacity: Option<u32>,
    remaining_energy: Option<u32>,
    present_rate: Option<u32>,
    voltage: Option<u32>,
    design_capacity: Option<u32>,
    design_capacity_unit: Option<u32>,
    last_capacity: Option<u32>,
    last_capacity_unit: Option<u32>,
    hours: Option<u32>,
    minutes: Option<u32>,
    seconds: Option<u32>,
    percentage: Option<u32>,
    is_battery: bool,
    capacity_unit: String,
}

impl Default for PowerSupplyInfo {
    fn default() -> PowerSupplyInfo {
        PowerSupplyInfo { 
            remaining_capacity: None,
            remaining_energy: None,
            present_rate: None,
            voltage: None,
            design_capacity: None,
            design_capacity_unit: None,
            last_capacity: None,
            last_capacity_unit: None,
            hours: None,
            minutes: None,
            seconds: None,
            percentage: None,
            is_battery: false,
            capacity_unit: "mAh".to_string() 
        }
    }
}

fn main() -> std::io::Result<()> {
    let acpi_path = "/sys/class".to_string();
    let power_supply_path = acpi_path + "/power_supply";

    for entry in fs::read_dir(&power_supply_path)? {
        let resource = (entry?).file_name().into_string().unwrap();
        let mut supply = PowerSupplyInfo { ..Default::default() };
        println!("{:}", resource);
        let resource_path = format!("{}/{}", &power_supply_path, resource);
        for entry in fs::read_dir(&resource_path)? {
            let resource_filename = (entry?).file_name().into_string().unwrap();
            println!("  {:}", resource_filename); 
            let entry_path = format!("{}/{}", &resource_path, resource_filename);
            match get_file_from_path(&entry_path) {
               Ok(mut f) => {
                   let mut contents = String::new();
                   f.read_to_string(&mut contents)?;
                   match resource_filename.as_ref() {
                       "type" => {
                           println!("Found the type!"); 
                           if contents == "battery" {
                               supply.is_battery = true;
                           }
                       },
                       _ => {},
                   }
               },
               Err(_e) => { continue; },
            }
        }
        if supply.is_battery {
            println!("{} {}", "Battery", 1);
        }
    }
    Ok(())
}

fn get_file_from_path(path: &String) -> std::result::Result<fs::File, &'static str> {
    let metadata = fs::metadata(&path).unwrap();
    let file_type = metadata.file_type();

    if file_type.is_dir() {
        return Err("File is a directory");
    } else if file_type.is_symlink() {
        return Err("File is a symlink");
    } else {
        return match fs::File::open(path) {
            Ok(f) => { Ok(f) },
            Err(e) => {
                eprintln!("Error: {}", e);
                Err("Could not open file")
            },
        };
    }
}
