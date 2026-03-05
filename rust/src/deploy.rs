mod utils;

use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use tracing::{error, info};
use utils::env::DeployConfig;
use utils::log::init_logger;

fn main() {
    // Load and validate environment
    let config = DeployConfig::from_env().expect("Failed to load deployment configuration");

    // Initialize logger
    init_logger("info", false);

    info!("Starting deployment to {}", config.deploy_host);

    match run_deployment(&config) {
        Ok(output) => {
            info!("Deployment completed successfully");
            println!("{}", output);
        }
        Err(e) => {
            error!(error = %e, "Deployment failed");
            std::process::exit(1);
        }
    }
}

fn run_deployment(config: &DeployConfig) -> Result<String, Box<dyn std::error::Error>> {
    // Read the generated config file
    let config_path = Path::new("config/sing-box.server.json");
    if !config_path.exists() {
        return Err(
            "Config file not found. Run 'cargo run --bin generate-config' first.".into(),
        );
    }

    let config_content = std::fs::read_to_string(config_path)?;

    // Connect to SSH
    let tcp = TcpStream::connect(format!("{}:22", config.deploy_host))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Authenticate with key
    sess.userauth_pubkey_file(&config.deploy_user, None, Path::new(&config.deploy_keyfile), None)?;

    // Create remote directory if it doesn't exist
    let mkdir_cmd = format!("mkdir -p {}", config.deploy_cwd);
    let mut channel = sess.channel_session()?;
    channel.exec(&mkdir_cmd)?;
    let mut output = String::new();
    channel.read_to_string(&mut output)?;
    channel.wait_close()?;

    // Upload config file using scp
    let remote_config_path = format!("{}/sing-box.server.json", config.deploy_cwd);
    let mut channel = sess.scp_send(Path::new(&remote_config_path), 0o644, config_content.len() as u64, None)?;
    channel.write_all(config_content.as_bytes())?;
    channel.send_eof()?;
    channel.wait_eof()?;
    channel.close()?;
    channel.wait_close()?;

    info!("Config file uploaded to {}", remote_config_path);

    // Execute deploy command if specified
    if !config.deploy_command.is_empty() {
        let command = format!("cd {} && {}", config.deploy_cwd, config.deploy_command);
        let mut channel = sess.channel_session()?;
        channel.exec(&command)?;

        // Read output
        let mut output = String::new();
        channel.read_to_string(&mut output)?;

        // Wait for channel to close
        channel.wait_close()?;

        if channel.exit_status()? != 0 {
            let mut stderr = String::new();
            channel.stderr().read_to_string(&mut stderr)?;
            return Err(format!("Command failed: {}", stderr).into());
        }

        info!("Deploy command executed successfully");
        return Ok(output);
    }

    Ok("Config uploaded successfully. Restart sing-box manually.".to_string())
}
