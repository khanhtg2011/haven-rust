use clap::{Parser, Subcommand};
use dashmap::DashMap;
use haven_core::{
    db::create_pool,
    models::PublicConfig,
};
use haven_server::app::{create_router, AppState};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Parser, Debug)]
#[command(
    name = "haven",
    author = "Khanhs <khanhtg2011@gmail.com>",
    version = "4.29.0",
    about = "Haven - Self-hosted private chat for your server, your rules",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the Haven chat server
    Server {
        #[arg(short, long, default_value = "3000")]
        port: u16,
        #[arg(short = 'H', long, default_value = "0.0.0.0")]
        host: String,
        #[arg(short, long, default_value = "./data/haven.db")]
        db: String,
        #[arg(long, default_value = "haven_default_secret_change_me_in_prod")]
        jwt_secret: String,
        #[arg(long, default_value = "./public")]
        public_dir: String,
    },
    /// Run the Haven Desktop client subsystems
    Desktop,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Server {
            port,
            host,
            db,
            jwt_secret,
            public_dir,
        } => {
            if let Some(parent) = std::path::Path::new(&db).parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            tracing::info!(
                host = %host,
                port = port,
                db = %db,
                public_dir = %public_dir,
                "Initializing Haven SQLite database & static assets"
            );
            let pool = create_pool(&db)?;

            let (tx_broadcast, _) = broadcast::channel(1024);
            let state = Arc::new(AppState {
                pool,
                jwt_secret: jwt_secret.into_bytes(),
                public_config: Arc::new(RwLock::new(PublicConfig::default())),
                voice_rooms: DashMap::new(),
                tx_broadcast,
                public_dir,
            });

            let app = create_router(state);
            let addr = format!("{}:{}", host, port);
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            tracing::info!("⬡ Haven server listening on http://{}", addr);

            axum::serve(listener, app).await?;
        }
        Commands::Desktop => {
            tracing::info!("Starting Haven Desktop client engine");
            let _hotkeys = haven_desktop::hotkeys::HotkeyManager::new();
            let overlay = haven_desktop::overlay::VoiceOverlayManager::default();
            let _audio = haven_desktop::audio::AudioCaptureService::new();

            tracing::info!(
                audio_supported = haven_desktop::audio::AudioCaptureService::is_supported(),
                overlay_position = %overlay.config.position,
                "Haven Desktop subsystems initialized"
            );
        }
    }

    Ok(())
}
