use clap::{Parser, Subcommand};

/// A lightweight and asynchronous Text User Interface(TUI) database visualizer.
#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Command,

    #[arg(short = 'C', long, default_value_t = 5)]
    pub max_connections: u32,

    #[arg(short = 'c', long, default_value_t = 1)]
    pub min_connections: u32,

    #[arg(short = 's', long, default_value_t = 25)]
    pub page_size: u8,
}

#[derive(Subcommand)]
pub enum Command {
    Sqlite {
        #[arg[short, long]]
        filename: String,
    },
    Mysql {
        #[arg(short, long, default_value_t = String::from("root"))]
        username: String,

        #[arg(short, long, default_value_t = String::new())]
        password: String,

        #[arg(short = 'H', long, default_value_t = String::from("localhost"))]
        host: String,

        #[arg(short = 'P', long, default_value_t = 3306)]
        port: u16,

        database: String,
    },
    Mariadb {
        #[arg(short, long, default_value_t = String::from("root"))]
        username: String,

        #[arg(short, long, default_value_t = String::new())]
        password: String,

        #[arg(short = 'H', long, default_value_t = String::from("localhost"))]
        host: String,

        #[arg(short = 'P', long, default_value_t = 3306)]
        port: u16,

        database: String,
    },
}
