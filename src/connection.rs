use std::path::PathBuf;

use crate::messages::client;
use crate::messages::server::{self, ReceivedMessage};
use anyhow::Result;
use log::{debug, error, trace};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::UnixStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;

pub struct Server {
    pub path: PathBuf,
    reader_handle: JoinHandle<()>,
    sender_handle: JoinHandle<()>,
    pub event_tx: Sender<client::Message>,
}

impl Server {
    pub async fn new(socket_path: PathBuf) -> Result<Server> {
        let stream = UnixStream::connect(&socket_path).await?;

        let (rx, tx) = stream.into_split();
        let reader = BufReader::new(rx);
        let writer = BufWriter::new(tx);

        // start the sender
        let (event_tx, event_rx): (Sender<client::Message>, Receiver<client::Message>) =
            mpsc::channel(10);
        let sender_handle = start_sender(event_rx, writer).await;
        trace!("sender running");

        // Start the listen loop
        let reader_handle = start_listener(reader, event_tx.clone()).await;
        trace!("listener running");

        Ok(Server {
            path: socket_path,
            event_tx,
            reader_handle,
            sender_handle,
        })
    }

    pub async fn send(&mut self, message: client::Message) -> Result<()> {
        self.event_tx.send(message).await?;
        Ok(())
    }

    pub async fn wait(self) -> Result<()> {
        tokio::select! {
            _ = self.sender_handle => Ok(()),
            _ = self.reader_handle => Ok(()),
        }
    }
}

async fn start_listener(
    reader: BufReader<OwnedReadHalf>,
    sender: Sender<client::Message>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut lines = reader.lines();
        while let Ok(Some(response)) = lines.next_line().await {
            debug!(" receiving < {}", response.trim());

            let parsed_response = match server::parse(response) {
                Ok(r) => r,
                Err(e) => {
                    // this is weird because we just notify the user that the message sucked and
                    // then happily move along to the next one
                    error!("Couldn't parse incomming message {e}");
                    continue;
                }
            };

            if let Err(e) = handle_response(&parsed_response, sender.clone()).await {
                error!("handling response {:?}, {e}", parsed_response);
            }
        }
    })
}

async fn handle_response(message: &ReceivedMessage, sender: Sender<client::Message>) -> Result<()> {
    match message {
        ReceivedMessage::Greeting(_g) => {
            sender.send(client::capabilities(1)).await?;
        }
        ReceivedMessage::Return(r) => trace!("received return value {r:#?}"),
    }
    Ok(())
}

async fn start_sender(
    mut events: Receiver<client::Message>,
    mut writer: BufWriter<OwnedWriteHalf>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            trace!("sending event");
            match event.encode() {
                Ok(payload) => {
                    trace!("sending {payload}");
                    // if we can't write to the socket that's a catastrophic error
                    writer.write_all(payload.as_bytes()).await.unwrap();
                    writer.flush().await.unwrap();
                }
                Err(e) => {
                    error!("couldn't encode event {e}");
                }
            }
        }
        trace!("send events channel closed");
    })
}
