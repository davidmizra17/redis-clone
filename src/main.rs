#![allow(unused_imports)]
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::Write;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.

    // Uncomment this block to pass the first stage
    
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buffer = [0; 512];
                loop {
                    let bytes_read = stream.read(&mut buffer).unwrap();
                    if bytes_read == 0 {
                        break;
                    }
                    stream.write(b"+PONG\r\n").unwarap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
