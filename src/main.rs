//! margin - Entry point for the desktop email client

use margin::App;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting margin");

    // Run the gpui application
    if let Err(e) = App::run() {
        tracing::error!("Application error: {}", e);
        std::process::exit(1);
    }
}
