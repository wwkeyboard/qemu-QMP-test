use std::io::{prelude::*, BufReader, BufWriter};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

use crate::messages::server::{self, ReceivedMessage};
use anyhow::Result;
use log::{debug, error, trace};

pub struct Server {
    pub path: PathBuf,
    pub reader_thread: JoinHandle<()>,
    pub writer: BufWriter<UnixStream>,
}

impl Server {
    pub fn new(socket_path: String) -> Result<Server, std::io::Error> {
        let bind_path = PathBuf::from(socket_path);
        let stream = UnixStream::connect(&bind_path)?;
        let reader = BufReader::new(stream.try_clone().expect("Couldn't clone socket"));
        let writer = BufWriter::new(stream);

        let listen_handle = listen(reader);

        Ok(Server {
            path: bind_path,
            reader_thread: listen_handle,
            writer,
        })
    }

    pub fn send(&mut self, message: String) -> Result<(), std::io::Error> {
        debug!(" sending   > {}", message.trim());
        let amount = self.writer.write(message.as_bytes())?;
        trace!("sent {} bytes", amount);
        self.writer.flush()?;
        Ok(())
    }
}

fn listen(mut reader: BufReader<UnixStream>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let mut response = String::new();
            let _len = reader
                .read_line(&mut response)
                .expect("couldn't read from socket");
            debug!(" receiving < {}", response.trim());

            // TODO: break these apart, we need to decide where the
            // split is between the thread that reads and parses
            // messages, and the thread(s) that handle the responses
            match parse_response(response) {
                Ok(r) => handle_response(r),
                Err(e) => {
                    error!("Couldn't parse incomming message {e}");
                }
            };
        }
    })
}

fn parse_response(data: String) -> Result<ReceivedMessage> {
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
