use std::{env, io};

use tokio::net::TcpListener;

mod app;
mod artists;
mod bands;
mod db;
mod events;
mod geo;
mod http;
mod i18n;
mod images;
mod map_pin;
mod ui;
mod venues;
mod videos;
#[tokio::main]
async fn main() {
    env_logger::init();
    db::initialize_database()
        .await
        .expect("initialize the database");

    let host = host_from_env().unwrap();
    let port = port_from_env().unwrap();

    let listener = TcpListener::bind(format!("{host}:{port}")).await.unwrap();

    println!("Listening on http://{}", listener.local_addr().unwrap());

    //Concurrent tasks, the first one to exit will cause the program to stop
    tokio::select! {
        result = topcoat::serve(listener, app::build_application_router()) => result.unwrap(),
        _ = goodbye() => ()
    }
}

fn host_from_env() -> Result<String, io::Error> {
    const HOST_ENV: &str = "HOST";
    const DEFAULT_HOST: &str = "127.0.0.1";

    match env::var(HOST_ENV) {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Ok(DEFAULT_HOST.to_owned()),
        Err(error) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{HOST_ENV} must be valid Unicode: {error}"),
        )),
    }
}

fn port_from_env() -> Result<u16, io::Error> {
    const PORT_ENV: &str = "PORT";
    const DEFAULT_PORT: u16 = 3000;

    match env::var(PORT_ENV) {
        Ok(value) => value.parse().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{PORT_ENV} must be a valid port number: {error}"),
            )
        }),
        Err(env::VarError::NotPresent) => Ok(DEFAULT_PORT),
        Err(error) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{PORT_ENV} must be valid Unicode: {error}"),
        )),
    }
}

pub async fn goodbye() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for Ctrl+C");

    println!("Goodbye!");
}
