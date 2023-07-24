use std::error::Error;
use std::io::{prelude::*, BufReader, BufWriter};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, thread};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().into_iter();

    args.next(); // yeet the path of this executable
    let socket_path = args.next().ok_or("must provide socket path")?;
    println!("opening {:?}", socket_path);
    let bind_path = PathBuf::from(socket_path);

    let stream = UnixStream::connect(bind_path)?;

    println!("listening");

    let mut reader = BufReader::new(stream.try_clone().expect("Couldn't clone socket"));

    thread::spawn(move || {
        loop {
            let mut response = String::new();
            let len = reader.read_line(&mut response).expect("couldn't read from socket");
            println!("{len}: {response}");
         }
    });

    let mut writer = BufWriter::new(stream);
    writer.write_fmt(format_args!(r#"{{"execute": }}"#))?;
    writer.flush()?;

    thread::sleep(Duration::from_millis(10000));

    Ok(())
}
