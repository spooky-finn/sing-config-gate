//! Deployment utility for deploying sing-box configuration to remote servers.
//!
//! This binary connects to multiple servers via SSH in parallel and executes
//! a deployment command on each one.

use std::{
    io::prelude::*,
    net::TcpStream,
    path::Path,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use ssh2::Session;
use tracing::{error, info};

use sing_box_config_bot::{
    config::get_env,
    errors::DeployError,
    utils::logger,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerNode {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
}

impl ServerNode {
    /// Returns the SSH address in the format `host:port`.
    pub fn ssh_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Returns the SSH address with username in the format `user@host`.
    pub fn ssh_user_address(&self) -> String {
        format!("{}@{}", self.user, self.host)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub keyfile: String,
    pub deploy_user: String,
    pub deploy_command: String,
    pub deploy_cwd: String,
    pub servers: Vec<ServerNode>,
}

impl DeployConfig {
    pub fn from_env() -> Result<Self, DeployError> {
        Ok(Self {
            keyfile: get_env("DEPLOY_KEYFILE").map_err(|e| DeployError::Ssh(e.to_string()))?,
            deploy_user: get_env("DEPLOY_USER").map_err(|e| DeployError::Ssh(e.to_string()))?,
            deploy_command: get_env("DEPLOY_COMMAND")
                .map_err(|e| DeployError::Ssh(e.to_string()))?,
            deploy_cwd: get_env("DEPLOY_CWD").map_err(|e| DeployError::Ssh(e.to_string()))?,
            servers: Self::load_servers()?,
        })
    }

    fn load_servers() -> Result<Vec<ServerNode>, DeployError> {
        // Load servers from DEPLOY_SERVERS environment variable
        // Format: "name1:user1:host1:port1,name2:user2:host2:port2,..."
        let servers_str = get_env("DEPLOY_SERVERS").map_err(|e| DeployError::Ssh(e.to_string()))?;

        let servers: Result<Vec<ServerNode>, DeployError> = servers_str
            .split(',')
            .map(|s| {
                let parts: Vec<&str> = s.split(':').collect();
                if parts.len() != 4 {
                    return Err(DeployError::Ssh(format!(
                        "Expected name:user:host:port, got: {}",
                        s
                    )));
                }

                let port = parts[3].parse::<u16>().map_err(|_| {
                    DeployError::Ssh(format!("Invalid port: {}", parts[3]))
                })?;

                Ok(ServerNode {
                    name: parts[0].to_string(),
                    user: parts[1].to_string(),
                    host: parts[2].to_string(),
                    port,
                })
            })
            .collect();

        servers
    }
}

fn main() {
    let config = DeployConfig::from_env().expect("Failed to load deployment configuration");
    logger::init("info", false);

    info!("Starting deployment to {} server(s)", config.servers.len());

    let config = Arc::new(config);
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut handles = vec![];

    // Deploy to all servers in parallel
    for server in config.servers.clone() {
        let config = Arc::clone(&config);
        let results = Arc::clone(&results);

        let handle = std::thread::spawn(move || {
            info!("Deploying to {} ({})", server.name, server.host);

            let output = run_deployment(&config, &server);

            let mut results = results.lock().unwrap();
            results.push((server.name.clone(), output.is_ok()));

            match output {
                Ok(out) => {
                    info!("Deployment to {} completed successfully", server.name);
                    println!("\n=== {} ===\n{}", server.name, out);
                }
                Err(e) => {
                    error!(
                        server = %server.name,
                        error = %e,
                        "Deployment failed"
                    );
                    eprintln!("\n=== {} ===\nDeployment failed: {}", server.name, e);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all deployments to complete
    for handle in handles {
        handle.join().expect("Deployment thread panicked");
    }

    // Summary
    let results = results.lock().unwrap();
    let success_count = results.iter().filter(|(_, ok)| *ok).count();
    let fail_count = results.iter().filter(|(_, ok)| !*ok).count();

    println!(
        "\n=== Deployment Summary ===\n{}: {}\n{}: {}",
        "Successful", success_count, "Failed", fail_count
    );

    if fail_count > 0 {
        std::process::exit(1);
    }
}

fn run_deployment(
    config: &DeployConfig,
    server: &ServerNode,
) -> Result<String, DeployError> {
    let stream = TcpStream::connect(server.ssh_address())?;
    let mut session = Session::new()?;
    session.set_tcp_stream(stream);
    session.handshake()?;

    session.userauth_pubkey_file(&config.deploy_user, None, Path::new(&config.keyfile), None)?;

    let mut channel = session.channel_session()?;

    // Change to working directory and execute command
    let command = format!("cd {} && {}", config.deploy_cwd, config.deploy_command);
    channel.exec(&command)?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;
    channel.wait_eof()?;
    channel.close()?;
    channel.wait_close()?;

    let exit_status = channel.exit_status()?;
    if exit_status != 0 {
        return Err(DeployError::CommandFailed(exit_status, output));
    }

    Ok(output)
}
