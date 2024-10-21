use app_config::AppConfig;
use clap::{Parser, Subcommand};
use daemon_client::{Requester, SocketClient};
use output::Output;
use process::execute_tail;
use project_settings::ProjectSettings;

mod output;
mod process;

/// Simple process manager
#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create new project or replaces existing
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
    Rm {
        /// name of the project
        project: String,
    },
}

fn main() {
    match run() {
        Output::Stdout(res) => {
            println!("{}", res);
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

        Command::Rm { project } => requester.remove_project(&project).into(),

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
        } => {
            let res = match service {
                Some(service) => requester
                    .get_services_info(&project, &service)
                    .map(|res| vec![res.value.logfile_path]),
                None => requester.get_project_info(&project).map(|res| {
                    res.value
                        .services
                        .into_iter()
                        .map(|val| val.logfile_path)
                        .collect()
                }),
            };

            match res {
                Ok(res) => {
                    if raw {
                        Output::Stdout(res.join(" "))
                    } else {
                        let error = execute_tail(res);
                        // Should never reach here because execute replaces executable
                        Output::Stderr(error.to_string())
                    }
                }
                Err(err) => err.into(),
            }
        }
    }
}
