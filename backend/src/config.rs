use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub markdown_dir: PathBuf,
    pub jwt_secret: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| String::from("md_wiki.db"));
        
        let markdown_dir = PathBuf::from(
            std::env::var("MARKDOWN_DIR")
                .unwrap_or_else(|_| String::from("storage/markdown_files")),
        );

        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| String::from("your-secret-key"));

        let server_port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        Self {
            database_url,
            markdown_dir,
            jwt_secret,
            server_port,
        }
    }
} 