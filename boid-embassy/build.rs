use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
    embuild::espidf::sysenv::output();

    // Read WiFi configuration from cfg.toml
    let config_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("cfg.toml");

    if !config_path.exists() {
        panic!(
            "cfg.toml not found!\n\
            Please copy cfg.toml.example to cfg.toml and set your WiFi credentials:\n\
            cp cfg.toml.example cfg.toml"
        );
    }

    let config_str = fs::read_to_string(&config_path)
        .expect("Failed to read cfg.toml");

    // Parse TOML
    let config: toml::Value = config_str.parse()
        .expect("Failed to parse cfg.toml");

    // Extract WiFi credentials
    let wifi_ssid = config
        .get("wifi")
        .and_then(|w| w.get("ssid"))
        .and_then(|s| s.as_str())
        .expect("Missing wifi.ssid in cfg.toml");

    let wifi_password = config
        .get("wifi")
        .and_then(|w| w.get("password"))
        .and_then(|s| s.as_str())
        .expect("Missing wifi.password in cfg.toml");

    // Validate credentials are not placeholders
    if wifi_ssid == "YourNetworkName" || wifi_ssid.is_empty() {
        panic!(
            "Please set a valid WiFi SSID in cfg.toml\n\
            Current value: '{}'\n\
            Edit cfg.toml and set your actual network name",
            wifi_ssid
        );
    }

    if wifi_password == "YourPassword" || wifi_password.is_empty() {
        panic!(
            "Please set a valid WiFi password in cfg.toml\n\
            Edit cfg.toml and set your actual WiFi password"
        );
    }

    // Set environment variables for compile time
    println!("cargo:rustc-env=WIFI_SSID={}", wifi_ssid);
    println!("cargo:rustc-env=WIFI_PASSWORD={}", wifi_password);

    // Rebuild if cfg.toml changes
    println!("cargo:rerun-if-changed=cfg.toml");
}
