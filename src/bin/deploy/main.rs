use std::{
    env,
    process::{Command, Stdio},
};
use tracing::{error, info};

use sing_box_config_bot::utils::logger;

#[derive(Debug)]
struct DeployConfig {
    ssh_alias: String,
    deploy_path: String,
    service_name: String,
    sudo_password: Option<String>,
}

impl DeployConfig {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::from_filename("src/bin/deploy/.env.deploy").ok();
        Ok(Self {
            ssh_alias: env::var("DEPLOY_SSH_ALIAS")?,
            deploy_path: env::var("DEPLOY_PATH")?,
            service_name: env::var("SERVICE_NAME").unwrap_or_else(|_| "sing-box".to_string()),
            sudo_password: env::var("DEPLOY_SUDO_PASSWORD").ok(),
        })
    }
}

fn main() {
    logger::init("info", true);
    let config = DeployConfig::load().unwrap();
    info!(
        "Starting deployment. Target: {} -> {}",
        config.ssh_alias, config.deploy_path
    );
    if let Err(e) = deploy(&config) {
        error!("Deployment failed: {}", e);
        std::process::exit(1);
    }
    info!("Deployment completed successfully!");
}

fn deploy(config: &DeployConfig) -> Result<(), Box<dyn std::error::Error>> {
    info!("Pulling latest changes from git...");
    git_pull(config)?;

    info!("Building project on server...");
    build(config)?;

    info!("Restarting service...");
    restart_service(config)?;

    Ok(())
}

fn git_pull(config: &DeployConfig) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("ssh")
        .arg(&config.ssh_alias)
        .arg(format!(
            "cd {} && git fetch && git reset --hard origin/main",
            config.deploy_path
        ))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err("failed to pull latest changes".into());
    }
    Ok(())
}

fn build(config: &DeployConfig) -> Result<(), Box<dyn std::error::Error>> {
    let remote_cmd = format!(
        "cd {} && \
         source $HOME/.profile || true && \
         source $HOME/.bashrc || true && \
         rustup update && \
         cargo build --release --bin bot",
        config.deploy_path
    );

    let status = Command::new("ssh")
        .arg(&config.ssh_alias)
        .arg(remote_cmd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("build failed on server".into())
    }
}

fn restart_service(config: &DeployConfig) -> Result<(), Box<dyn std::error::Error>> {
    let remote_cmd = if let Some(pass) = &config.sudo_password {
        format!(
            "echo '{}' | sudo -S systemctl restart {} && \
             echo '{}' | sudo -S systemctl status {} --no-pager",
            pass, config.service_name, pass, config.service_name
        )
    } else {
        format!(
            "sudo systemctl restart {} && sudo systemctl status {} --no-pager",
            config.service_name, config.service_name
        )
    };

    let status = Command::new("ssh")
        .arg(&config.ssh_alias)
        .arg(remote_cmd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("failed to restart service".into())
    }
}
