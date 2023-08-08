use std::env;

use anyhow::{Context, Result};
use qemu_qmp_test::connection::Server;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let socket_path = path()?;

    let server = Server::new(socket_path).await?;

    server.wait().await?;

    Ok(())
}

fn path() -> Result<String> {
    let mut args = env::args();
    args.next();
    let socket_path = args.next().context("must provide socket path")?;
    Ok(socket_path)
}
