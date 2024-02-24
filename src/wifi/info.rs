use regex::Regex;
use std::{error::Error, process::Command};

pub fn get_ssid() -> Result<String, Box<dyn Error>> {
    // Windows
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        let cmd = Command::new("netsh")
            .args(["wlan", "show", "interfaces"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()?;

        if !cmd.status.success() {
            return Err(format!(
                "Command didn't run succesfully. Status code: {}.",
                cmd.status.code().unwrap_or(-1)
            )
            .into());
        }

        let stdout = String::from_utf8(cmd.stdout)?.replace("\r\n", "\n");
        let re = Regex::new(r"(?m)^\ +?SSID\ +?:\ +?(?<ssid>.*?)$")?;
        let Some(captures) = re.captures(stdout.as_str()) else {
            return Err("Couldn't get SSID.".into());
        };

        return Ok(captures["ssid"].to_string());
    }
    
    
    #[cfg(not(target_os = "windows"))]
    unimplemented!("Only Windows is supported at the moment.");
}
