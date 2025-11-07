use crate::args::{Args, Command};
use color_eyre::Result;
use sqlx::{
    MySql, Pool, Sqlite,
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::time::Duration;

pub enum Database {
    SQLite(Pool<Sqlite>),
    MySQL(Pool<MySql>),
}

impl Database {
    pub async fn connect(args: &Args) -> Result<Self> {
        Ok(match &args.subcommand {
            Command::Sqlite { filename } => {
                let conn = SqliteConnectOptions::new().filename(filename);
                Self::SQLite(
                    SqlitePoolOptions::new()
                        .min_connections(args.min_connections)
                        .max_connections(args.max_connections)
                        .connect_with(conn)
                        .await?,
                )
            }
            Command::Mysql {
                username,
                password,
                host,
                port,
                database,
            }
            | Command::Mariadb {
                username,
                password,
                host,
                port,
                database,
            } => {
                let conn = MySqlConnectOptions::new()
                    .username(username)
                    .password(password)
                    .host(host)
                    .port(*port)
                    .database(database);

                Self::MySQL(
                    MySqlPoolOptions::new()
                        .min_connections(args.min_connections)
                        .max_connections(args.max_connections)
                        .acquire_timeout(Duration::from_millis(100))
                        .connect_with(conn)
                        .await?,
                )
            }
        })
    }
}
