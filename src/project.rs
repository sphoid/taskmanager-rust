use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fs::File;
use std::str::FromStr;
use std::error::Error;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::collections::HashMap;
use clap::{Subcommand, Args};

use crate::cli::RuntimeConfig;

const PROJECTS_FILE: &str = "projects.json";

#[derive(Debug, Args, Clone)]
pub struct ProjectArgs {
	#[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectTaskType {
	Default,
}

impl FromStr for ProjectTaskType {
	type Err = ();

	fn from_str(input: &str) -> Result<ProjectTaskType, Self::Err> {
		match input {
			"default" => Ok(ProjectTaskType::Default),
			_         => Err(()),
		}
	}
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectTaskStatus {
	Default,
	Todo,
	InProgress,
	Complete,
}

impl FromStr for ProjectTaskStatus {
	type Err = ();

	fn from_str(input: &str) -> Result<ProjectTaskStatus, Self::Err> {
		match input {
			"todo"       => Ok(ProjectTaskStatus::Todo),
			"in_progress" => Ok(ProjectTaskStatus::InProgress),
			"complete"   => Ok(ProjectTaskStatus::Complete),
			_            => Ok(ProjectTaskStatus::Default),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectTask {
	pub id: Uuid,
	pub name: String,
	pub description: String,
	pub type_: ProjectTaskType,
	pub status: ProjectTaskStatus,
}

impl ProjectTask {
	fn new(name: &str, description: &str, type_: &str, status: &str) -> Self {
		let task_type_result = ProjectTaskType::from_str(type_);
		let task_type = match task_type_result {
			Ok(task_type) => task_type,
			Err(_) => ProjectTaskType::Default,
		};
		let task_status_result = ProjectTaskStatus::from_str(status);
		let task_status = match task_status_result {
			Ok(task_status) => task_status,
			Err(_) => ProjectTaskStatus::Todo,
		};

		Self {
			id: Uuid::new_v4(),
			name: name.to_string(),
			description: description.to_string(),
			type_: task_type,
			status: task_status,
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
	pub id: Uuid,
	pub name: String,
	pub description: String,
	pub tasks: HashMap<Uuid, ProjectTask>,
}

impl Project {
	pub fn new(name: &String, description: &String) -> Self {
		Self {
			id: Uuid::new_v4(),
			name: name.to_string(),
			description: description.to_string(),
			tasks: HashMap::new(),
		}
	}

	pub fn create_task(&mut self, name: &String, description: &String) -> Uuid {
		let task = ProjectTask::new(name, description, "default", "todo");
		let task_id = task.id.clone();

		self.tasks.insert(task_id, task);

		task_id
	}

	pub fn destroy_task(&mut self, task_id: &Uuid) -> Result<bool, Box<dyn Error>> {
		self.tasks.remove(task_id).unwrap();

		Ok(true)
	}
}

#[derive(Debug, Clone)]
pub struct ProjectData {
	projects: HashMap<Uuid, Project>,
}

impl ProjectData {
	pub fn create_project(&mut self, name: &String, description: &String) -> Uuid {
		let project = Project::new(name, description);
		let project_id = project.id.clone();
		self.projects.insert(project_id, project);

		project_id
	}

	pub fn destroy_project(&mut self, project_id: &Uuid) -> Result<bool, Box<dyn Error>> {
		self.projects.remove(project_id).unwrap();

		Ok(true)
	}
}

fn load_projects() -> Result<HashMap<Uuid, Project>, Box<dyn Error>> {
	if !Path::new(PROJECTS_FILE).exists() {
        return Ok(HashMap::new());
    }
	let file = File::open(PROJECTS_FILE)?;
    let reader = BufReader::new(file);
    let projects = serde_json::from_reader(reader)?;

	Ok(projects)
}

pub fn load_data() -> Result<ProjectData, Box<dyn Error>> {
	let projects = load_projects()?;

	Ok(ProjectData { projects })
}

pub fn write_data(data: &ProjectData) -> Result<(), Box<dyn Error>> {
	let file = File::create(PROJECTS_FILE)?;
	serde_json::to_writer(file, &data.projects)?;

	Ok(())
}

pub fn run_command(rtc: &mut RuntimeConfig, args: &ProjectArgs) -> Result<(), Box<dyn Error>> {
	let project_command = &args.command.clone().unwrap();

	match project_command {
		Command::Create { name, description } => {
			let project_description = match description {
				Some(description) => description,
				None => &"".to_string(),
			};
			rtc.projects_data.create_project(&name, &project_description);
		},
		Command::Destroy { project_id } => {
			let project_uuid = Uuid::parse_str(project_id.as_str())?;

			rtc.projects_data.destroy_project(&project_uuid)?;
		},
		Command::Update { project_id, name, description } => {
			let project_uuid = Uuid::parse_str(project_id.as_str())?;
			let project = rtc.projects_data.projects.get_mut(&project_uuid).ok_or_else(|| {
				io::Error::new(io::ErrorKind::NotFound, "Project not found")
			})?;

			if let Some(name) = name {
				project.name = name.clone();
			}
			if let Some(description) = description {
				project.description = description.clone();
			}
		},
		Command::List => {
			let projects = &rtc.projects_data.projects;

			println!("Projects:");
			for (project_id, project) in projects {
				println!("{}: {} - {}", project_id, project.name, project.description);
			}
		},
		Command::CreateTask { project_id, name, description } => {
			let project_uuid = Uuid::parse_str(project_id.as_str())?;
			let task_description = match description {
				Some(description) => description,
				None => &"".to_string(),
			};
			let project = match rtc.projects_data.projects.get_mut(&project_uuid) {
				Some(project) => project,
				None => {
					return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Project not found")));
				},
			};

			project.create_task(&name, &task_description);
		},
		Command::DestroyTask { project_id, task_id } => {
			let project_uuid = Uuid::parse_str(project_id.as_str())?;
			let task_uuid = Uuid::parse_str(task_id.as_str())?;
			let project = match rtc.projects_data.projects.get_mut(&project_uuid) {
				Some(project) => project,
				None => {
					return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Project not found")));
				},
			};

			project.destroy_task(&task_uuid)?;
		},
		Command::UpdateTask { project_id, task_id, name, description } => {
			let project_uuid = Uuid::parse_str(project_id.as_str())?;
			let task_uuid = Uuid::parse_str(task_id.as_str())?;
			let project = match rtc.projects_data.projects.get_mut(&project_uuid) {
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
		Command::ListTasks { project_id } => {
			let project_uuid = Uuid::parse_str(project_id.as_str())?;
			let project = match rtc.projects_data.projects.get_mut(&project_uuid) {
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