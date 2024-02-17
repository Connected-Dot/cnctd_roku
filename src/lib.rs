use cnctd_rest::Rest;
use anyhow::Result;
use std::{collections::HashSet, fs::{self, File}, io::Write, path::PathBuf};
use serde_json::Value;
use chrono::Local;
use regex::Regex;
use cnctd_shell::Shell;
use telnet::{Telnet, Event};

pub struct Roku {
    pub url: String,
    pub log_path: String,
}

impl Roku {
    pub fn new(url: &str, log_path: Option<&str>) -> Self {
        Self {
            url: url.to_string(),
            log_path: if log_path.is_some() { log_path.unwrap().into() } else { "~/Documents/logs/roku".into() }
        }
    }

    pub async fn command(&self, key: &str) -> Result<()> {
        let valid_commands: HashSet<&str> = [
            "Home", "Rev", "Fwd", "Play", "Select", "Left", "Right", 
            "Down", "Up", "Back", "InstantReplay", "Info", 
            "Backspace", "Search", "Enter"
        ].iter().cloned().collect();

        if valid_commands.contains(key) {
            let _response: Value = Rest::post(&format!("{}/keypress/{}", &self.url, key), "").await?;
            println!("Sending command: {:?}", key);
            Ok(())
        } else {
            // Handle unrecognized command
            Err(anyhow::Error::msg("Unrecognized command"))
        }
    }

    pub async fn get_player(&self) -> Result<()> {
        let response: Value = Rest::get(&format!("{}/query/media-player", &self.url)).await?;
        
        Ok(())
    }

    // pub async fn log(&self) -> Result<()> {
    //     let home_dir = dirs::home_dir().expect("Could not find home directory");
    //     let expanded_log_path = if self.log_path.starts_with("~/") {
    //         home_dir.join(&self.log_path[2..])
    //     } else {
    //         PathBuf::from(&self.log_path)
    //     };
    
    //     // Ensure the log directory exists
    //     fs::create_dir_all(&expanded_log_path)?;

    //     let log_url = Regex::new(r"https?://([^:/]+):\d+")?
    //         .replace(&self.url, "$1 8050")
    //         .to_string();
    
    //     // Replace colons in the timestamp to avoid issues with filenames
    //     let ts = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    //     let full_path = format!("{}/roku_{}.txt", expanded_log_path.to_str().unwrap(), ts);
    //     let command = format!("telnet {} > {}", log_url, full_path);
    //     let exit_code = Shell::run_with_exit_status(&command, true).await?;
    //     println!("exit: {}", exit_code);

    //     Ok(())
    // }

    pub async fn log(&self) -> Result<()> {
        // Ensure the log directory exists
        std::fs::create_dir_all(&self.log_path)?;
    
        // Create and open the log file
        let ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let log_file_path = format!("{}/roku_{}.txt", &self.log_path, ts);
        let mut file = File::create(log_file_path)?;
    
        // Create a Telnet connection
        let mut connection = Telnet::connect(("192.168.1.174", 8085), 256)?;
    
        // Read events from the Telnet connection
        loop {
            match connection.read_nonblocking() {
                Ok(Event::Data(data)) => {
                    println!("Received data: {:?}", data);
                    file.write_all(&data)?;
                },
                Ok(Event::TimedOut) => break, // Or handle timeout appropriately
                Ok(_) => {}, // Handle other events as needed
                Err(e) => return Err(anyhow::Error::new(e)),
            }
        }
    
        Ok(())
    }
    
}
