fn main() -> std::io::Result<()> {
    let _power_supplies: Vec<acpi_client::PowerSupplyInfo> =
        match acpi_client::get_power_supply_info() {
            Ok(ps) => ps,
            Err(e) => {
                eprintln!("Application error: {}", e);
                std::process::exit(1);
            }
        };

    Ok(())
}
