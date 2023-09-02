use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::messages::client::{self, Message};
use crate::messages::server::{self, ReceivedMessage, Return};
use anyhow::Result;
use log::{debug, error, trace};
use tokio::io::AsyncBufReadExt;
use tokio::io::{AsyncRead, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::UnixStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Server tracks the connection with the QEMU QMP server.
///
/// This manages the sender and receiver to the QMP socket. It takes care of
/// serializing communication so we don't intermix messages. It tracks callbacks
/// to run when QEMU sends a response to a command.
///
/// next_message_id tracks the next safe message ID to send to with a message
/// that expects a response.
///
///
pub struct Server {
    pub path: PathBuf,
    reader_handle: JoinHandle<()>,
    sender_handle: JoinHandle<()>,
    pub event_tx: Sender<Message>,
    callback_db: CallBackDB,
    next_message_id: AtomicUsize,
}

type CallBackDB = Arc<Mutex<HashMap<usize, Box<dyn Fn(Return) + Send + 'static>>>>;

impl Server {
    /// binds to the socket at `socket_path`, creates some internal tracking data
    /// structures, starts the sender and receiver.
    pub async fn bind(socket_path: PathBuf) -> Result<Server> {
        let stream = UnixStream::connect(&socket_path).await?;

        let (socket_rx, socket_tx) = stream.into_split();

        let callback_db: CallBackDB = Arc::new(Mutex::new(HashMap::new()));
        // start the sender
        let (event_tx, event_rx): (Sender<Message>, Receiver<Message>) = mpsc::channel(10);

        // start the sender loop
        let writer = BufWriter::new(socket_tx);
        let sender_handle = start_sender(event_rx, writer).await;
        trace!("sender running");

        // Start the listen loop
        let reader = BufReader::new(socket_rx);
        let reader_handle = start_listener(reader, event_tx.clone(), callback_db.clone()).await;
        trace!("listener running");

        Ok(Server {
            path: socket_path,
            event_tx,
            reader_handle,
            sender_handle,
            callback_db,
            next_message_id: AtomicUsize::new(1),
        })
    }

    pub async fn send(&mut self, message: Message) -> Result<()> {
        self.event_tx.send(message).await?;
        Ok(())
    }

    pub async fn call<T>(&mut self, message: Message, callback: T) -> Result<()>
    where
        T: Fn(Return) + Send + 'static,
    {
        let mut db = self.callback_db.lock().await;
        let id = self.next_id();
        db.insert(id, Box::new(callback));

        let mut payload = message.clone();
        payload.id = id;

        self.event_tx.send(message).await?;

        Ok(())
    }

    pub async fn wait(self) -> Result<()> {
        tokio::select! {
            _ = self.sender_handle => Ok(()),
            _ = self.reader_handle => Ok(()),
        }
    }

    fn next_id(&self) -> usize {
        self.next_message_id.fetch_add(1, Ordering::Relaxed)
    }
}

async fn start_listener<T>(
    reader: BufReader<T>,
    sender: Sender<Message>,
    cb_db: CallBackDB,
) -> JoinHandle<()>
where
    T: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = reader.lines();
        while let Ok(Some(response)) = lines.next_line().await {
            debug!(" receiving < {}", response.trim());

            let parsed_response = match server::parse(response) {
                Ok(r) => r,
                Err(e) => {
                    // this is weird because we just notify the user
                    // that the message sucked and then happily move
                    // along to the next one
                    error!("Couldn't parse incomming message {e}");
                    continue;
                }
            };

            if let Err(e) = handle_response(parsed_response, sender.clone(), cb_db.clone()).await {
                error!("handling response {e}");
            }
        }
    })
}

async fn handle_response(
    message: ReceivedMessage,
    sender: Sender<Message>,
    cb_db: CallBackDB,
) -> Result<()> {
    match message {
        ReceivedMessage::Greeting(_g) => {
            sender.send(client::capabilities()).await?;
        }
        ReceivedMessage::Return(r) => {
            trace!("received return value {r:#?}");
            if let Some(id) = r.id {
                trace!("running callback for {id}");
                let mut db = cb_db.lock().await;
                if let Some(cb) = db.remove(&id) {
                    cb(*r);
                };
            }
        }
        ReceivedMessage::Event(event) => {
            trace!("received event {event:?}");
        }
    }
    Ok(())
}

async fn start_sender<T: AsyncWriteExt + Unpin + Send + 'static>(
    mut events: Receiver<Message>,
    mut writer: BufWriter<T>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
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
