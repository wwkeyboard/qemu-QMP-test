use std::io::{prelude::*, BufReader, BufWriter};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::thread::{JoinHandle, self};

use crate::messages::server::{ReceivedMessage, self};

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
            writer: writer,
        })
    }

    pub fn send(&mut self, message: String) -> Result<(), std::io::Error>{
        self.writer.write(message.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }
}

fn listen(mut reader: BufReader<UnixStream>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let mut response = String::new();
            let len = reader.read_line(&mut response).expect("couldn't read from socket");

            println!("{len}: {response}");
         }
    })
}

fn parse_response(data: String) -> Result<ReceivedMessage, String> {
    server::parse(data).or(Err("parsing response from server".into()))
}