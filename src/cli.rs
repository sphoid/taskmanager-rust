use std::error::Error;
use std::io;
use clap::{Parser, Subcommand, Args};
use uuid::Uuid;
use taskmanager::project;
use taskmanager::config;

#[derive(Parser, Debug)]
#[command(name = "taskmanager-cli")]
#[command(version = "1.0")]
#[command(about = "Manage projects and tasks", long_about = None)]
pub struct Cli {
	#[command(subcommand)]
	namespace: Namespace,
}


#[derive(Debug, Args, Clone)]
pub struct ProjectArgs {
	#[command(subcommand)]
    command: Option<ProjectCommand>,
}

#[derive(Debug, Subcommand, Clone)]
enum ProjectCommand {
	Create {
		name: String,
		description: Option<String>,
	},
	Destroy {
		project_id: String,
	},
	Update {
		project_id: String,
		#[arg(long)]
		name: Option<String>,
		#[arg(long)]
		description: Option<String>,
	},
	List,
	CreateTask {
		project_id: String,
		name: String,
		description: Option<String>,
	},
	DestroyTask {
		project_id: String,
		task_id: String,
	},
	UpdateTask {
		project_id: String,
		task_id: String,
		#[arg(long)]
		name: Option<String>,
		#[arg(long)]
		description: Option<String>,
	},
	ListTasks {
		project_id: String,
	},
}

#[derive(Debug, Args, Clone)]
pub struct ConfigArgs {
	#[command(subcommand)]
    command: Option<ConfigCommand>,
}

#[derive(Debug, Subcommand, Clone)]
enum ConfigCommand {
	Get {
		key: String,
	},
	Set {
		key: String,
		value: String,
	}
}

#[derive(Debug, Subcommand, Clone)]
pub enum Namespace {
	Project(ProjectArgs),
	Config(ConfigArgs),
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
	pub namespace: Namespace,
	pub config: config::Config,
	pub projects_data: project::ProjectData,
}

impl RuntimeConfig {
	pub fn build() -> Result<RuntimeConfig, Box<dyn Error>> {
		let projects_data = project::load_data()?;
		let config = config::load_config()?;

		let cli = Cli::parse();
		match &cli.namespace {
			Namespace::Project(_) => {
				Ok(RuntimeConfig { namespace: cli.namespace.clone(), config, projects_data })
			},
			Namespace::Config(_) => {
				Ok(RuntimeConfig { namespace: cli.namespace.clone(), config, projects_data })
			}
		}
	}

	pub fn persist(&self) -> Result<(), Box<dyn Error>> {
		project::write_data(&self.projects_data)
	}

	pub fn run_config_command(&mut self, args: &ConfigArgs) -> Result<(), Box<dyn Error>> {
		let config_command = &args.command.clone().unwrap();

		match config_command {
			ConfigCommand::Get { key } => {
				match key.as_str() {
					"persistence_mode" => {
						println!("Persistence Mode: {:?}", &self.config.persistence_mode);
					},
					_ => {
						println!("Invalid config key");
					}
				};
			},
			ConfigCommand::Set { key, value } => {
				println!("Setting config key: {} to value: {}", key, value);
			},
		}

		Ok(())
	}

	pub fn run_project_command(&mut self, args: &ProjectArgs) -> Result<(), Box<dyn Error>> {
		let project_command = &args.command.clone().unwrap();

		match project_command {
			ProjectCommand::Create { name, description } => {
				let project_description = match description {
					Some(description) => description,
					None => &"".to_string(),
				};
				self.projects_data.create_project(&name, &project_description);
			},
			ProjectCommand::Destroy { project_id } => {
				let project_uuid = Uuid::parse_str(project_id.as_str())?;

				self.projects_data.destroy_project(&project_uuid)?;
			},
			ProjectCommand::Update { project_id, name, description } => {
				let project_uuid = Uuid::parse_str(project_id.as_str())?;
				let project = self.projects_data.get_project_mut(&project_uuid).ok_or_else(|| {
					io::Error::new(io::ErrorKind::NotFound, "Project not found")
				})?;

				if let Some(name) = name {
					project.name = name.clone();
				}
				if let Some(description) = description {
					project.description = description.clone();
				}
			},
			ProjectCommand::List => {
				let projects = &self.projects_data.get_projects();

				println!("Projects:");
				for project in projects {
					println!("{}: {} - {}", project.id, project.name, project.description);
				}
			},
			ProjectCommand::CreateTask { project_id, name, description } => {
				let project_uuid = Uuid::parse_str(project_id.as_str())?;
				let task_description = match description {
					Some(description) => description,
					None => &"".to_string(),
				};
				let project = match self.projects_data.get_project_mut(&project_uuid) {
					Some(project) => project,
					None => {
						return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Project not found")));
					},
				};

				project.create_task(&name, &task_description);
			},
			ProjectCommand::DestroyTask { project_id, task_id } => {
				let project_uuid = Uuid::parse_str(project_id.as_str())?;
				let task_uuid = Uuid::parse_str(task_id.as_str())?;
				let project = match self.projects_data.get_project_mut(&project_uuid) {
					Some(project) => project,
					None => {
						return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Project not found")));
					},
				};

				project.destroy_task(&task_uuid)?;
			},
			ProjectCommand::UpdateTask { project_id, task_id, name, description } => {
				let project_uuid = Uuid::parse_str(project_id.as_str())?;
				let task_uuid = Uuid::parse_str(task_id.as_str())?;
				let project = match self.projects_data.get_project_mut(&project_uuid) {
					Some(project) => project,
					None => {
						return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Project not found")));
					},
				};
				let task = match project.tasks.get_mut(&task_uuid) {
					Some(task) => task,
					None => {
						return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Task not found")));
					},
				};

				if let Some(name) = name {
					task.name = name.clone();
				}
				if let Some(description) = description {
					task.description = description.clone();
				}
			},
			ProjectCommand::ListTasks { project_id } => {
				let project_uuid = Uuid::parse_str(project_id.as_str())?;
				let project = match self.projects_data.get_project(&project_uuid) {
					Some(project) => project,
					None => {
						return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Project not found")));
					},
				};
				println!("Project tasks:");

				for (task_id, task) in &project.tasks {
					println!("{}: {} - {}", task_id, task.name, task.description);
				}
			}
		}

		Ok(())
	}

	pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
		println!("Running taskmanager-cli lib version: {}", taskmanager::get_lib_version());
		let namespace = self.namespace.clone();
		match namespace {
			Namespace::Project(args) => {
				let run_result = self.run_project_command(&args);
				match run_result {
					Ok(_) => {
						self.persist()?;

						Ok(())
					},
					Err(err) => Err(err),
				}
			},
			Namespace::Config(args) => {
				let run_result = self.run_config_command(&args);
				match run_result {
					Ok(_) => {
						self.persist()?;

						Ok(())
					},
					Err(err) => Err(err),
				}
			}
		}
	}
}
