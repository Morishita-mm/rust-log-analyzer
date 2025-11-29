mod app;
mod state;
mod tui;
mod types;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    app::run().await
}
