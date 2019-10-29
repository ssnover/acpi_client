use std::fs;

fn main() -> std::io::Result<()> {
    for entry in fs::read_dir("/sys/class/power_supply")? {
        let dir = entry?;
        println!("{:}", dir.file_name().into_string().unwrap());
    }
    Ok(())
}
