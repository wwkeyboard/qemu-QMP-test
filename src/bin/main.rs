use anyhow::Result;
use qemu_qmp_test::{connection::Server, cli::Args, cli::Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::load_from_cli();

    pretty_env_logger::init();

    let server = Server::new(args.path).await?;

    if let Some(Commands::Send{payload}) = args.command {
        println!("--- {payload}");
    }

    server.wait().await?;

    Ok(())
}
