#[cfg(test)]
mod tests {
    #[test]
    fn verify_mock_file_coulomb_parse() {
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();
        let mock_path = dir.path().join("BAT1");
        let _mock_adapter = std::fs::create_dir(&mock_path).unwrap();
        let mut file = std::fs::File::create(&mock_path.join("charge_full")).unwrap();
        writeln!(file, "2000000").unwrap();
        let mut file = std::fs::File::create(&mock_path.join("charge_full_design")).unwrap();
        writeln!(file, "2800000").unwrap();
        let mut file = std::fs::File::create(&mock_path.join("charge_now")).unwrap();
        writeln!(file, "1000000").unwrap();
        let mut file = std::fs::File::create(&mock_path.join("current_now")).unwrap();
        writeln!(file, "599000").unwrap();
        let mut file = std::fs::File::create(&mock_path.join("status")).unwrap();
        writeln!(file, "Discharging").unwrap();
        let mut file = std::fs::File::create(&mock_path.join("type")).unwrap();
        writeln!(file, "Battery").unwrap();
        let mut file = std::fs::File::create(&mock_path.join("voltage_now")).unwrap();
        writeln!(file, "15045000").unwrap();

        let batteries = acpi_client::get_battery_info(&dir.path());
        assert!(batteries.is_ok());
        assert_eq!(batteries.unwrap().len(), 1);

        drop(file);
        dir.close().unwrap();
    }
}
