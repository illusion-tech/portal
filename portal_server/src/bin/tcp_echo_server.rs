use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

pub fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("[::1]:8080")?;
    println!("Server listening on [::1]:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    let mut buf = [0; 1024];

                    loop {
                        let bytes_read = match stream.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => n,
                            Err(e) => {
                                eprintln!("Error reading from stream: {}", e);
                                break;
                            }
                        };
                        if let Err(e) = stream.write_all(&buf[..bytes_read]) {
                            eprintln!("Error writing to stream: {}", e);
                            break;
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}
