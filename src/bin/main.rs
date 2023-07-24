use std::error::Error;
use std::io::{prelude::*, BufReader, BufWriter};
use std::time::Duration;
use std::{env, thread};

use qemu_qmp_test::connection::Server;

fn main() -> Result<(), Box<dyn Error>> {
    let socket_path = path()?;
    
    let mut server = Server::new(socket_path)?;

    println!("listening");

    thread::spawn(move || {
        loop {
            let mut response = String::new();
            let len = server.reader.read_line(&mut response).expect("couldn't read from socket");
            println!("{len}: {response}");
         }
    });

    server.writer.write_fmt(format_args!(r#"{{"execute": }}"#))?;
    server.writer.flush()?;

    thread::sleep(Duration::from_millis(10000));

    Ok(())
}

fn path() -> Result<String, Box<dyn Error>> {
    let mut args = env::args().into_iter();
    args.next();
    let socket_path = args.next().ok_or("must provide socket path")?;
    Ok(socket_path)
}
