use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_url: String,
    pub token: String,
    pub hostname: String,
    pub docker_path: String,
    pub github_repo: Option<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let server_url = env::var("SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let token = env::var("CLIENT_TOKEN")
            .map_err(|_| anyhow::anyhow!("CLIENT_TOKEN environment variable is required"))?;

        let hostname = env::var("HOSTNAME")
            .or_else(|_| hostname::get().map(|h| h.to_string_lossy().to_string()))
            .unwrap_or_else(|_| "unknown".to_string());

        let docker_path = env::var("DOCKER_PATH")
            .unwrap_or_else(|_| "/var/lib/docker".to_string());

        let github_repo = env::var("GITHUB_REPO").ok();

        Ok(Self {
            server_url,
            token,
            hostname,
            docker_path,
            github_repo,
        })
    }
}
