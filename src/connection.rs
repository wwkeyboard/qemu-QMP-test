use std::io::{prelude::*, BufReader, BufWriter};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use crate::messages::client;
use crate::messages::server::{self, ReceivedMessage};
use anyhow::Result;
use log::{debug, error, trace};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;

pub struct Server {
    pub path: PathBuf,
    pub reader_thread: JoinHandle<()>,
    pub event_tx: Sender<client::Message>,
}

impl Server {
    pub async fn new(socket_path: String) -> Result<Server> {
        let bind_path = PathBuf::from(socket_path);
        let stream = UnixStream::connect(&bind_path)?;

        // Grabs the first message, which should be a Greeting. Sends
        // a response about oob capabilities.
        // unwrap because if this fails we have to bail the program
        negotiate_capabilities(stream.try_clone()?)?;

        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream);

        let (event_tx, event_rx): (Sender<client::Message>, Receiver<client::Message>) =
            mpsc::channel(10);
        start_sender(event_rx, writer).await?;

        // Start the listen loop
        let listen_handle = listen(reader).await;

        Ok(Server {
            path: bind_path,
            reader_thread: listen_handle,
            event_tx,
        })
    }

    pub async fn send(&mut self, message: client::Message) -> Result<()> {
        self.event_tx.send(message).await?;
        Ok(())
    }
}

fn negotiate_capabilities(stream: UnixStream) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone().expect("Couldn't clone socket"));
    let mut writer = BufWriter::new(stream);

    let mut response = String::new();
    let _len = reader
        .read_line(&mut response)
        .expect("couldn't read from socket");

    debug!(" negotiating < {}", response.trim());

    let capabilities = client::capabilities().encode()?;
    debug!(" negotiating > {capabilities}");

    writer.write(capabilities.as_bytes())?;
    Ok(())
}

async fn listen(mut reader: BufReader<UnixStream>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut response = String::new();
        loop {
            let _len = reader
                .read_line(&mut response)
                .expect("couldn't read from socket");
            debug!(" receiving < {}", response.trim());

            // TODO: break these apart, we need to decide where the
            // split is between the thread that reads and parses
            // messages, and the thread(s) that handle the responses
            match parse_response(&response) {
                Ok(r) => handle_response(r),
                Err(e) => {
                    error!("Couldn't parse incomming message {e}");
                }
            };
        }
    })
}

fn parse_response(data: &String) -> Result<ReceivedMessage> {
    trace!(" parsing   : {}", data.clone().trim());

    server::parse(data.clone())
}

fn handle_response(message: ReceivedMessage) {
    match message {
        ReceivedMessage::Greeting(_g) => {
            println!(">> received greeting ");
            // respond
        }
    }
}

async fn start_sender(
    mut events: Receiver<client::Message>,
    mut writer: BufWriter<UnixStream>,
) -> Result<()> {
    while let Some(event) = events.recv().await {
        let payload = event.encode()?;
        writer.write(payload.as_bytes())?;
    }
    Ok(())
}
