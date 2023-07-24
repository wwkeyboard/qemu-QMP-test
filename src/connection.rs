use std::io::{BufReader, BufWriter};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

pub struct Server {
    pub path: PathBuf,
    pub reader: BufReader<UnixStream>,
    pub writer: BufWriter<UnixStream>,
}

impl Server {
    pub fn new(socket_path: String) -> Result<Server, std::io::Error> {
        let bind_path = PathBuf::from(socket_path);
        let stream = UnixStream::connect(&bind_path)?;
        let reader = BufReader::new(stream.try_clone().expect("Couldn't clone socket"));
        let writer = BufWriter::new(stream);

        Ok(Server {
            path: bind_path,
            reader: reader,
            writer: writer,
        })
    }
}
