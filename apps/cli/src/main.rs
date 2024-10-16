use std::process::exit;

use clap::{Parser, Subcommand};
use config::CliConfig;
use daemon_client::{Requester, SocketClient};
use output::Output;
use process::execute_tail;
use project_settings::ProjectSettings;

mod config;
mod output;
mod process;

/// Simple process manager
#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Creates new project or replaces existing
    Upsert {
        /// path to the settings file or directory containing the settings file (conc.json), defaults to current dir
        settings_path: Option<String>,
    },
    /// Get status of all projects, single project or a service
    Ps {
        /// name of the project
        project_name: Option<String>,
        /// name of the service
        service_name: Option<String>,
    },
    /// Open logs for a project or a service in 'tail -f' command
    Logs {
        /// name of the project
        project_name: String,
        /// name of the service
        service_name: Option<String>,
        /// if specified programs returns logfile path(s) instead of running tail on them
        #[clap(long, short, action)]
        raw: bool,
    },
    /// Start a project or a service
    Start {
        /// name of the project
        project_name: String,
        /// name of the service
        service_name: Option<String>,
    },
    /// Restart a project or a service
    Restart {
        /// name of the project
        project_name: String,
        /// name of the service
        service_name: Option<String>,
    },
    /// Stop a project or a service
    Stop {
        /// name of the project
        project_name: String,
        /// name of the service
        service_name: Option<String>,
    },
    /// Remove a project
    Rm {
        /// name of the project
        project_name: String,
    },
}

fn main() {
    match application() {
        Output::Stdout(res) => {
            println!("{}", res);
            exit(0);
        }
        Output::Stderr(res) => {
            eprintln!("{}", res);
            exit(-1);
        }
    }
}

fn application() -> Output {
    let config = match CliConfig::new() {
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
        Command::Upsert { settings_path } => {
            let settings = ProjectSettings::find_parse_and_populate(settings_path);

            match settings {
                Ok(settings) => {
                    let json = String::try_from(&settings);
                    match json {
                        Ok(json) => requester.upsert_project(&json).into(),
                        Err(err) => err.into(),
                    }
                }
                Err(err) => err.into(),
            }
        }
        Command::Ps {
            project_name,
            service_name,
        } => match project_name {
            Some(project_name) => match service_name {
                Some(service_name) => requester
                    .get_services_info(&project_name, &service_name)
                    .into(),
                None => requester.get_project_info(&project_name).into(),
            },
            None => requester.get_projects_info().into(),
        },
        Command::Logs {
            project_name,
            service_name,
            raw,
        } => {
            let res = match service_name {
                Some(service_name) => requester
                    .get_services_info(&project_name, &service_name)
                    .map(|res| vec![res.value.logfile_path]),
                None => requester.get_project_info(&project_name).map(|res| {
                    res.value
                        .services
                        .into_iter()
                        .map(|val| val.logfile_path)
                        .collect::<Vec<String>>()
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
        Command::Start {
            project_name,
            service_name,
        } => match service_name {
            Some(service_name) => requester.start_service(&project_name, &service_name).into(),
            None => requester.start_project(&project_name).into(),
        },
        Command::Restart {
            project_name,
            service_name,
        } => match service_name {
            Some(service_name) => requester
                .restart_service(&project_name, &service_name)
                .into(),
            None => requester.restart_project(&project_name).into(),
        },
        Command::Stop {
            project_name,
            service_name,
        } => match service_name {
            Some(service_name) => requester.stop_service(&project_name, &service_name).into(),
            None => requester.stop_project(&project_name).into(),
        },
        Command::Rm { project_name } => requester.remove_project(&project_name).into(),
    }
}
