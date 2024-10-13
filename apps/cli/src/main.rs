use std::process::exit;

use clap::{Parser, Subcommand};
use daemon_client::{Requester, SocketClient};
use output::Outputable;
use project_settings::ProjectSettings;

mod output;

/// Simple process manager
#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Creates new project or replaces existing
    Upsert,
    /// Get status of all projects, single project or a service
    Ps {
        /// name of the project
        project_name: Option<String>,
        /// name of the service
        service_name: Option<String>,
    },
    /// Starts a project or a service
    Start {
        /// name of the project
        project_name: String,
        /// name of the service
        service_name: Option<String>,
    },
    /// Stops a project or a service
    Stop {
        /// name of the project
        project_name: String,
        /// name of the service
        service_name: Option<String>,
    },
    /// Removes a project
    Rm {
        /// name of the project
        project_name: String,
    },
}

fn main() {
    let parsed = Cli::parse();

    let socket_client = SocketClient::new("/home/prixladi/rep/conc/apps/daemon/run/conc.sock");
    if !socket_client.is_alive() {
        eprintln!("Cannot connect to the Conc daemon at unix://{}.\nDaemon is not running or is using different work directory.", socket_client.socket_path);
        exit(-2)
    }
    let requester = Requester::new(&socket_client);

    let output = match parsed.command {
        Commands::Upsert => {
            let json = "";
            let settings = ProjectSettings::try_from(json);

            match settings {
                Ok(res) => println!("{}", res.name),
                Err(e) => eprintln!("Error while trying to deserialize settings: {}", e),
            }
            requester
                .upsert_project(&json)
                .unwrap()
                .map(|e| e.to_output())
        }
        Commands::Ps {
            project_name,
            service_name,
        } => match project_name {
            Some(p_name) => match service_name {
                Some(s_name) => requester
                    .get_services_info(&p_name, &s_name)
                    .unwrap()
                    .map(|e| e.to_output()),
                None => requester
                    .get_project_info(&p_name)
                    .unwrap()
                    .map(|e| e.to_output()),
            },
            None => requester
                .get_projects_info()
                .unwrap()
                .map(|e| e.to_output()),
        },
        Commands::Start {
            project_name,
            service_name,
        } => match service_name {
            Some(s_name) => requester
                .start_service(&project_name, &s_name)
                .unwrap()
                .map(|e| e.to_output()),
            None => requester
                .start_project(&project_name)
                .unwrap()
                .map(|e| e.to_output()),
        },
        Commands::Stop {
            project_name,
            service_name,
        } => match service_name {
            Some(s_name) => requester
                .stop_service(&project_name, &s_name)
                .unwrap()
                .map(|e| e.to_output()),
            None => requester
                .stop_project(&project_name)
                .unwrap()
                .map(|e| e.to_output()),
        },
        Commands::Rm { project_name } => requester
            .remove_project(&project_name)
            .unwrap()
            .map(|e| e.to_output()),
    };

    let was_success = output.is_ok();
    let output_str = output.unwrap_or_else(|e| e.to_output());

    match was_success {
        true => {
            println!("{}", output_str);
            exit(0);
        }
        false => {
            eprintln!("{}", output_str);
            exit(-1);
        }
    }
}
