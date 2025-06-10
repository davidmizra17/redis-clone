#![allow(unused_imports)]
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
mod resp;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.

    // Uncomment this block to pass the first stage
    
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Listening on 127.0.0.1:6379");

    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((mut stream, client)) => {
                println!("new accepted connection from client {:?}", client);
                tokio::spawn(async move {
                    handle_conn(stream).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    
               
    }

    async fn handle_conn(stream: TcpStream) {
        let  _handler = resp::RespHandler::new(stream);
    }

