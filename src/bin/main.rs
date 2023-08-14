use anyhow::Result;
use qemu_qmp_test::{connection::Server, cli::Args};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::load_from_cli();

    pretty_env_logger::init();

    let server = Server::new(args.path).await?;

    server.wait().await?;

    Ok(())
}
