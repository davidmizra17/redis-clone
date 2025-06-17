#![allow(unused_imports)]
use anyhow::Result;
use resp::Value;
use std::collections::HashMap;
use std::{collections::hash_map, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

mod resp;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Listening on 127.0.0.1:6379");

    loop {
        let shared_state = Arc::new(Mutex::new(HashMap::new()));
        let stream = listener.accept().await;

        match stream {
            Ok((mut stream, client)) => {
                println!("new accepted connection from client {:?}", client);
                tokio::spawn(async move {
                    handle_conn(stream, shared_state.clone()).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_conn(stream: TcpStream, hash_map: Arc<Mutex<HashMap<String, String>>>) {
    let mut handler = resp::RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        println!("Got value {:?}", value);
        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                "get" => {
                    let key = if let Some(Value::BulkString(k)) = args.get(0) {
                        k
                    } else {
                        ""
                    };
                    let map = hash_map.lock().await;
                    match map.get(key) {
                        Some(val) => Value::BulkString(val.clone()),
                        None => Value::BulkString("".to_string()),
                    }
                }
                "set" => {
                    if let (Some(Value::BulkString(key)), Some(Value::BulkString(val))) =
                        (args.get(0), args.get(1))
                    {
                        let mut map = hash_map.lock().await;
                        map.insert(key.clone(), val.clone());
                        Value::SimpleString("OK".to_string())
                    } else {
                        Value::SimpleString("ERR".to_string())
                    }
                }
                c => panic!("Cannot handle command {}", c),
            }
        } else {
            break;
        };
        println!("Sending value {:?}", response);

        handler.write_value(response).await.unwrap();
    }
}
fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => Ok((
            unpack_bulk_str(a.first().unwrap().clone())?,
            a.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected command to be a bulk string")),
    }
}
