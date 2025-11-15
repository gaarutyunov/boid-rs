use std::path::PathBuf;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() {
    // Check if cfg.toml exists
    let config_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("cfg.toml");

    if !config_path.exists() {
        panic!(
            "You need to create a `cfg.toml` file with your Wi-Fi credentials! Use `cfg.toml.example` as a template.\n\n\
            Example:\n\
            [wifi]\n\
            ssid = \"YourNetworkName\"\n\
            psk = \"YourPassword\"\n"
        );
    }

    // Validate that credentials are not placeholders
    if CONFIG.wifi_ssid.is_empty() || CONFIG.wifi_ssid == "YourNetworkName" {
        panic!(
            "Please set a valid WiFi SSID in cfg.toml\n\
            Current value: '{}'\n\
            Edit cfg.toml and set your actual network name",
            CONFIG.wifi_ssid
        );
    }

    if CONFIG.wifi_psk.is_empty() || CONFIG.wifi_psk == "YourPassword" {
        panic!(
            "Please set a valid WiFi password in cfg.toml\n\
            Edit cfg.toml and set your actual WiFi password"
        );
    }

    // Set environment variables for compile time
    println!("cargo:rustc-env=WIFI_SSID={}", CONFIG.wifi_ssid);
    println!("cargo:rustc-env=WIFI_PASSWORD={}", CONFIG.wifi_psk);

    // Rebuild if cfg.toml changes
    println!("cargo:rerun-if-changed=cfg.toml");

    // Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
    embuild::espidf::sysenv::output();
}
