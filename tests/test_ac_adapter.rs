#[cfg(test)]
mod tests {
    #[test]
    fn verify_mock_file_parse() {
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();
        let mock_path = dir.path().join("ACAD");
        let _mock_adapter = std::fs::create_dir(&mock_path).unwrap();
        let mut file = std::fs::File::create(&mock_path.join("type")).unwrap();
        writeln!(file, "Mains").unwrap();
        let mut file = std::fs::File::create(&mock_path.join("online")).unwrap();
        writeln!(file, "1").unwrap();

        let adapters = acpi_client::get_ac_adapter_info(&dir.path());
        assert!(adapters.is_ok());
        assert_eq!(adapters.unwrap().len(), 1);

        drop(file);
        dir.close().unwrap();
    }

    #[test]
    fn parse_mock_adapter() {
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();
        let mock_path = dir.path().join("ACAD");
        let _mock_adapter = std::fs::create_dir(&mock_path).unwrap();
        let mut file = std::fs::File::create(&mock_path.join("type")).unwrap();
        writeln!(file, "Mains").unwrap();
        let mut file = std::fs::File::create(&mock_path.join("online")).unwrap();
        writeln!(file, "1").unwrap();

        let acad = acpi_client::ACAdapterInfo::new(&mock_path).unwrap();
        assert_eq!(acad.name, String::from("ACAD"));
        assert_eq!(acad.status, acpi_client::Status::Online);

        drop(file);
        dir.close().unwrap();
    }
}
