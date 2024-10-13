use std::process::exit;

use clap::{Parser, Subcommand};
use daemon_client::{Requester, SocketClient};
use output::Output;
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

    let output: Output = match parsed.command {
        Commands::Upsert => {
            let json = "";
            let settings = ProjectSettings::try_from(json);

            match settings {
                Ok(res) => println!("{}", res.name),
                Err(e) => eprintln!("Error while trying to deserialize settings: {}", e),
            }
            requester.upsert_project(&json).into()
        }
        Commands::Ps {
            project_name,
            service_name,
        } => match project_name {
            Some(p_name) => match service_name {
                Some(s_name) => requester.get_services_info(&p_name, &s_name).into(),
                None => requester.get_project_info(&p_name).into(),
            },
            None => requester.get_projects_info().into(),
        },
        Commands::Start {
            project_name,
            service_name,
        } => match service_name {
            Some(s_name) => requester.start_service(&project_name, &s_name).into(),
            None => requester.start_project(&project_name).into(),
        },
        Commands::Stop {
            project_name,
            service_name,
        } => match service_name {
            Some(s_name) => requester.stop_service(&project_name, &s_name).into(),
            None => requester.stop_project(&project_name).into(),
        },
        Commands::Rm { project_name } => requester.remove_project(&project_name).into(),
    };

    match output {
        Output::Ok(res) => {
            println!("{}", res);
            exit(0);
        }
        Output::Err(res) => {
            eprintln!("{}", res);
            exit(-1);
        }
    }
}
