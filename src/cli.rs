// use std::str::FromStr;
use std::error::Error;
use clap::{Parser, Subcommand};
use crate::project;

#[derive(Parser, Debug)]
#[command(name = "taskmanager")]
#[command(version = "1.0")]
#[command(about = "Manage projects and tasks", long_about = None)]
pub struct Cli {
	#[command(subcommand)]
	namespace: Namespace,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Namespace {
	Project(project::ProjectArgs),
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
	pub namespace: Namespace,
	pub projects_data: project::ProjectData,
}

impl RuntimeConfig {
	pub fn build() -> Result<RuntimeConfig, Box<dyn Error>> {
		let cli = Cli::parse();
		match &cli.namespace {
			Namespace::Project(_) => {
				let projects_data = project::load_data()?;
				Ok(RuntimeConfig { namespace: cli.namespace.clone(), projects_data })
			},
		}
	}

	pub fn persist(&self) -> Result<(), Box<dyn Error>> {
		project::write_data(&self.projects_data)
	}

	pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
		let namespace = self.namespace.clone();
		match namespace {
			Namespace::Project(args) => {
				let run_result = project::run_command(self, &args);
				match run_result {
					Ok(_) => {
						self.persist()?;

						Ok(())
					},
					Err(err) => Err(err),
				}
			},
		}
	}
}
