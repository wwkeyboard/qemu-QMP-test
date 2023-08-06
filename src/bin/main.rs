use std::time::Duration;
use std::{env, thread};

use anyhow::{Context, Result};
use qemu_qmp_test::connection::Server;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let socket_path = path()?;

    let mut server = Server::new(socket_path)?;

    thread::sleep(Duration::from_millis(10000));

    // Sending a bad command to make sure the server yells back at us
    server.send(r#"{{"execute": }}"#.into())?;

    thread::sleep(Duration::from_millis(10000));

    Ok(())
}

fn path() -> Result<String> {
    let mut args = env::args();
    args.next();
    let socket_path = args.next().context("must provide socket path")?;
    Ok(socket_path)
}
