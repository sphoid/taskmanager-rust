use std::error::Error;
use std::path::Path;
use std::io::BufReader;
use serde::{Deserialize, Serialize};
use clap::{Subcommand, Args};

use crate::cli::RuntimeConfig;

const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Args, Clone)]
pub struct ConfigArgs {
	#[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
	Get {
		key: String,
	},
	Set {
		key: String,
		value: String,
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PersistenceMode {
	JSON,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
	persistence_mode: PersistenceMode,
}

impl Config {
	pub fn default() -> Self {
		Self {
			persistence_mode: PersistenceMode::JSON,
		}
	}
}

pub fn load_config() -> Result<Config, Box<dyn Error>> {
	let config_path = Path::new(CONFIG_FILE);
	if !config_path.exists() {
		return Ok(Config::default());
	}

	let file = std::fs::File::open(config_path)?;
	let reader = BufReader::new(file);
	let config: Config = serde_json::from_reader(reader)?;

	Ok(config)
}


pub fn run_command(rtc: &mut RuntimeConfig, args: &ConfigArgs) -> Result<(), Box<dyn Error>> {
	let config_command = &args.command.clone().unwrap();

	match config_command {
		Command::Get { key } => {
			match key.as_str() {
				"persistence_mode" => {
					println!("Persistence Mode: {:?}", &rtc.config.persistence_mode);
				},
				_ => {
					println!("Invalid config key");
				}
			};
		},
		Command::Set { key, value } => {
			println!("Setting config key: {} to value: {}", key, value);
		},
	}

	Ok(())
}