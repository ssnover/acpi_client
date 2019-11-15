# acpi-client

This project aims to create an executable in Rust which replaces the functionality of the acpitool provided by many Linux distributions allowing monitoring of laptop batteries, AC power supplies, and thermal systems and associated metadata.

Currently, this project supports only batteries, and only batteries that report information in terms of capacity (mAh) at that. Future work includes supporting batteries which report information in energy (mWh).

Additionally, this repository also sections off the workhorse part of the application away in a module so that other applications written in Rust can consume the information the tool can gather.

I'm still learning Rust, criticism of the source where I may not be following best practices is welcome and appreciated!

[![Crates.io][crates-badge]][crates-url]

[crates-badge]: https://img.shields.io/crates/v/acpi_client.svg
[crates-url]: https://crates.io/crates/acpi_client

