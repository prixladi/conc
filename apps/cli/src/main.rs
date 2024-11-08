use app_config::AppConfig;
use clap::{Parser, Subcommand};
use daemon_client::{Requester, SocketClient};
use interactive::interact;
use output::Output;
use process::execute_tail;
use project_settings::ProjectSettings;

mod interactive;
mod output;
mod process;
mod utils;

/// Simple process manager
#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Opens a interactive mode
    #[clap(visible_alias("i"))]
    Interactive,
    /// Create new project or replaces existing
    #[clap(visible_alias("up"))]
    Upsert {
        /// path to the settings file or directory containing the settings file (conc.json), defaults to current dir
        settings_path: Option<String>,
    },
    /// Get space delimited list of all project
    Projects,
    /// Get space delimited list of all service under project
    Services {
        /// name of the project
        project: String,
    },
    /// Get status of all projects, single project or a service
    Ps {
        /// name of the project
        project: Option<String>,
        /// name of the service
        service: Option<String>,
    },
    /// Open logs for a project or a service in 'tail -f' command
    Logs {
        /// name of the project
        project: String,
        /// name of the service
        service: Option<String>,
        /// if specified programs returns logfile path(s) instead of running tail on them
        #[clap(long, short, action)]
        raw: bool,
    },
    /// Start a project or a service
    Start {
        /// name of the project
        project: String,
        /// name of the service
        service: Option<String>,
    },
    /// Restart a project or a service
    Restart {
        /// name of the project
        project: String,
        /// name of the service
        service: Option<String>,
    },
    /// Stop a project or a service
    Stop {
        /// name of the project
        project: String,
        /// name of the service
        service: Option<String>,
    },
    /// Get project settings
    Settings {
        /// name of the project
        project: String,
    },
    /// Remove a project
    #[clap(visible_alias("rm"))]
    Remove {
        /// name of the project
        project: String,
    },
}

fn main() {
    match run() {
        Output::Stdout(res) => {
            if !res.is_empty() {
                println!("{}", res);
            }
            std::process::exit(0);
        }
        Output::Stderr(res) => {
            eprintln!("{}", res);
            std::process::exit(-1);
        }
    }
}

fn run() -> Output {
    let config = match AppConfig::new() {
        Ok(config) => config,
        Err(err) => return err.into(),
    };

    let cli = Cli::parse();

    let socket_client = SocketClient::new(&config.daemon_socket_path);
    if !socket_client.is_alive() {
        return Output::socket_not_alive(&socket_client.socket_path);
    }

    let requester = Requester::new(socket_client);
    match cli.command {
        Command::Interactive => interact(requester).into(),

        Command::Projects => requester.get_project_names().into(),

        Command::Services { project } => requester.get_service_names(&project).into(),

        Command::Ps {
            project: Some(project),
            service: Some(service),
        } => requester.get_services_info(&project, &service).into(),

        Command::Ps {
            project: Some(project),
            service: None,
        } => requester.get_project_info(&project).into(),

        Command::Ps {
            project: None,
            service: _,
        } => requester.get_projects_info().into(),

        Command::Start {
            project,
            service: Some(service),
        } => requester.start_service(&project, &service).into(),

        Command::Start {
            project,
            service: None,
        } => requester.start_project(&project).into(),

        Command::Restart {
            project,
            service: Some(service),
        } => requester.restart_service(&project, &service).into(),

        Command::Restart {
            project,
            service: None,
        } => requester.restart_project(&project).into(),

        Command::Stop {
            project,
            service: Some(service),
        } => requester.stop_service(&project, &service).into(),

        Command::Stop {
            project,
            service: None,
        } => requester.stop_project(&project).into(),

        Command::Settings { project } => requester.get_project_settings(&project).into(),

        Command::Remove { project } => requester.remove_project(&project).into(),

        Command::Upsert { settings_path } => {
            let settings = ProjectSettings::find_parse_and_populate(settings_path)
                .and_then(|settings| String::try_from(&settings));

            match settings {
                Ok(json) => requester.upsert_project(&json).into(),
                Err(err) => err.into(),
            }
        }

        Command::Logs {
            project,
            service,
            raw,
        } => match service {
            Some(service) => requester
                .get_services_info(&project, &service)
                .map(|service| vec![service.logfile_path]),
            None => requester.get_project_info(&project).map(|project| {
                project
                    .services
                    .into_iter()
                    .map(|val| val.logfile_path)
                    .collect()
            }),
        }
        .map(|res| match raw {
            true => Output::Stdout(res.join(" ")),
            false => {
                let error = execute_tail(res);
                Output::Stderr(error.to_string())
            }
        })
        .unwrap_or_else(|err| err.into()),
    }
}
