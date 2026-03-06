use std::{
    env,
    process::{Command, Stdio},
};

#[derive(Debug)]
struct DeployConfig {
    ssh_alias: String,
    deploy_path: String,
    service_name: String,
    docker_image: String,
    host_port: u16,
}

type Error = Box<dyn std::error::Error>;

impl DeployConfig {
    fn load() -> Result<Self, Error> {
        if let Err(e) = dotenvy::from_filename("src/bin/deploy/.env.deploy") {
            panic!("fail to load env .env.deploy: {}", e);
        }

        Ok(Self {
            ssh_alias: env::var("DEPLOY_SSH_ALIAS")?,
            deploy_path: env::var("DEPLOY_PATH")?,
            service_name: env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "sing-box-orchestrator".to_string()),
            docker_image: env::var("DOCKER_IMAGE")?,
            host_port: env::var("HOST_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()?,
        })
    }
}

fn main() {
    let config = DeployConfig::load().unwrap();
    if let Err(e) = deploy(&config) {
        eprintln!("Deployment failed: {}", e);
        std::process::exit(1);
    }
    println!("Deployment completed successfully!");
}

fn deploy(config: &DeployConfig) -> Result<(), Error> {
    fetch(config)?;
    build_image(config)?;
    start_container(config)?;
    Ok(())
}

fn fetch(config: &DeployConfig) -> Result<(), Error> {
    let remote_cmd = format!(
        "cd {deploy_path} && git pull",
        deploy_path = config.deploy_path,
    );

    let status = Command::new("ssh")
        .arg(&config.ssh_alias)
        .arg(remote_cmd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err("failed to checkout code on remote server".into());
    }
    Ok(())
}

fn build_image(config: &DeployConfig) -> Result<(), Error> {
    let remote_cmd = format!(
        "cd {deploy_path} && \
         docker build -t {docker_image} .",
        deploy_path = config.deploy_path,
        docker_image = config.docker_image,
    );

    let status = Command::new("ssh")
        .arg(&config.ssh_alias)
        .arg(remote_cmd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err("failed to build Docker image on remote server".into());
    }
    Ok(())
}

fn start_container(config: &DeployConfig) -> Result<(), Error> {
    let remote_cmd = format!(
        "cd {deploy_path} && \
         docker stop {service_name} 2>/dev/null || true && \
         docker rm {service_name} 2>/dev/null || true && \
         docker run -d \
           --name {service_name} \
           --restart unless-stopped \
           --env-file .env \
           -p {host_port}:8080 \
           -v ~/apps/vpn/data:/app/data \
           {docker_image}",
        deploy_path = config.deploy_path,
        service_name = config.service_name,
        docker_image = config.docker_image,
        host_port = config.host_port,
    );

    let status = Command::new("ssh")
        .arg(&config.ssh_alias)
        .arg(remote_cmd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err("failed to deploy to remote server".into());
    }
    Ok(())
}
