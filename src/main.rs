#![windows_subsystem = "windows"]
mod wifi;

use clokwerk::{AsyncScheduler, TimeUnits};
use config::{Config, File};
use futures::executor;
use simple_log::{error, warn, LogConfigBuilder};
use simple_log::{info, log_level};
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;
use std::{env, error::Error};
use wifi::info::get_ssid;

#[cfg(not(debug_assertions))]
use special_folder::SpecialFolder;

#[cfg(not(debug_assertions))]
use mslnk::ShellLink;

const CHECK_INTERVAL_SEC: u32 = 10;

const INDICATOR_COMMENT_START: &str = "#===[[ AUTO HOSTS UPDATER START";
const INDICATOR_COMMENT_END: &str = "#===[[ AUTO HOSTS UPDATER END";

const CONFIG_FILE: &str = "config.toml";
const DEFAULT_CONFIG: &str = r#"
hosts_path = 'C:\Windows\System32\drivers\etc\hosts'

[ssid]
example = """
    # Redirect requests to example.com to 192.168.1.1.
    192.168.1.1 example.com

    # Redirect requests to sub.example.com to 192.168.1.2.
    192.168.1.2 sub.example.com
"""
"#;

#[cfg(not(debug_assertions))]
fn add_to_startup(display_name: &str, path: &str) -> Result<(), Box<dyn Error>> {
    let startup_path = SpecialFolder::Startup.get().unwrap_or("".into());

    if startup_path.as_os_str().is_empty() {
        return Err("Couldn't get the startup path.".into());
    }

    let shortcut = ShellLink::new(path)?;

    if let Err(err) = shortcut.create_lnk(startup_path.join(format!("{}.lnk", display_name))) {
        Err(err.into())
    } else {
        Ok(())
    }
}

fn update_hosts() -> Result<(), Box<dyn Error>> {
    let ssid = get_ssid().unwrap_or("".into());

    let config = Config::builder()
        .add_source(File::with_name(CONFIG_FILE))
        .build()?;

    let hosts_path = config.get_string("hosts_path")?;
    let ssid_list = config.get_table("ssid")?;

    if !Path::new(&hosts_path).exists() {
        return Err(format!("Hosts file doesn't exist in given path: {}.", hosts_path).into());
    }

    let hosts_file = String::from_utf8(fs::read(&hosts_path)?)?;

    let mut lines: Vec<&str> = hosts_file.lines().collect();
    let indicator_start = lines.iter().position(|&e| e == INDICATOR_COMMENT_START);
    let indicator_end = lines.iter().position(|&e| e == INDICATOR_COMMENT_END);

    for (key, val) in ssid_list.iter() {
        if *key == ssid {
            let val = val.to_string();

            if val.is_empty() {
                warn!(r#"SSID "{}" has an empty value assigned. Skipping..."#, key);
                return Ok(());
            }

            let val = val
                .lines()
                .map(|e| e.trim())
                .collect::<Vec<&str>>()
                .join("\n");

            if let (Some(indicator_start), Some(indicator_end)) = (indicator_start, indicator_end) {
                lines.drain((indicator_start + 1)..indicator_end);
                lines.insert(indicator_start + 1, &val);
            } else {
                lines.push(INDICATOR_COMMENT_START);
                lines.push(&val);
                lines.push(INDICATOR_COMMENT_END);
            }

            let lines = lines.join("\n");

            if lines == hosts_file {
                return Ok(());
            }

            fs::write(&hosts_path, lines)?;
            info!(
                r#"Hosts file updated with the new value(s) for the SSID "{}"."#,
                ssid
            );
            return Ok(());
        }
    }

    // No value found with the associated SSID, clear the existing values.
    if let (Some(indicator_start), Some(indicator_end)) = (indicator_start, indicator_end) {
        lines.drain(indicator_start..=indicator_end);
        fs::write(&hosts_path, lines.join("\n"))?;

        if ssid == "" {
            info!("Not connected to any network. Clearing the existing value(s).");
        } else {
            info!(
                r#"No value found for the SSID "{}". Clearing the existing value(s)."#,
                ssid
            );
        }
    }

    Ok(())
}

fn log_error(f: fn() -> Result<(), Box<dyn Error>>) {
    if let Err(err) = f() {
        error!("{}", err);
    };
}

fn _main() -> Result<(), Box<dyn Error>> {
    simple_log::new(
        LogConfigBuilder::builder()
            .path("logs.txt")
            .level(log_level::INFO)
            .size(1)
            .roll_count(1)
            .output_file()
            .build(),
    )?;

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args.get(1).unwrap_or(&"".into()).as_str() {
            "ssid" => {
                info!("{}", get_ssid()?);
                return Ok(());
            }
            "version" => {
                info!("Version: v{}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {}
        }
    }

    #[cfg(not(debug_assertions))]
    {
        let bin_path = env::current_exe()?;
        let bin_path = bin_path.to_str().unwrap_or("");

        if !bin_path.is_empty() {
            add_to_startup("Hosts Updater", bin_path)?;
        } else {
            error!("Binary path is empty. Couldn't set the file as auto launch.");
        }
    }

    if !Path::new(CONFIG_FILE).exists() {
        fs::write(CONFIG_FILE, DEFAULT_CONFIG.trim())?;
    }

    log_error(update_hosts);

    executor::block_on(async {
        let mut scheduler = AsyncScheduler::new();

        scheduler.every(CHECK_INTERVAL_SEC.seconds()).run(|| async {
            log_error(update_hosts);
        });

        loop {
            scheduler.run_pending().await;
            async_std::task::sleep(Duration::from_secs(CHECK_INTERVAL_SEC.into())).await;
        }
    });

    Ok(())
}

fn main() -> ExitCode {
    if let Err(err) = _main() {
        error!("{}", err.to_string());
        return 1.into();
    }

    0.into()
}
