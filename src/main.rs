mod app;
mod args;
mod db;
mod layout;
mod widgets;

use app::App;
use args::Args;
use clap::Parser;
use color_eyre::Result;
use scopeguard::defer;

#[tokio::main]
async fn main() -> Result<()> {
    defer! {
        ratatui::restore();
    }

    color_eyre::install()?;

    let args = Args::parse();
    let terminal = ratatui::init();

    App::build(args).await?.run(terminal).await
}
