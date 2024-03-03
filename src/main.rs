use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug)]
enum Message {
    ClientConnected{
        author: Arc<TcpStream>
    },
    ClientDisconnected{
        author: Arc<TcpStream>
    },
    NewMessage {
        author: Arc<TcpStream>,
        bytes: Vec<u8>
    }
}

struct Client {
    conn: Arc<TcpStream>
}

fn server(message_receiver: Receiver<Message>) {

    let mut clients = HashMap::new();

    loop {

        let msg = message_receiver.recv().expect("The server receiver is not hung up");

        match msg {

            Message::ClientConnected{author} => {
                let addr = author.peer_addr().expect("TODO: Cache");
                clients.insert(addr.clone(), Client {
                    conn: author.clone()
                });
            }

            Message::ClientDisconnected{author} => {
                let addr = author.peer_addr().expect("TODO: Cache");
                clients.remove(&addr);
            }

            Message::NewMessage {author, bytes} => {
                let author_addr = author.peer_addr().expect("TODO: Cache");
                for (addr, client) in clients.iter() {
                    if *addr != author_addr {
                        let _ = client.conn.as_ref().write(&bytes);
                    }
                }
            }

        }

    }

}

fn handle_client(stream: Arc<TcpStream>, message_sender: Sender<Message>) {

    let _ = message_sender.send(Message::ClientConnected{author: stream.clone()}).map_err(|err| {
        eprintln!("ERR: could not send message to the server, {err}");
    });

    let mut buffer = Vec::new();
    buffer.resize(64, 0);

    writeln!(stream.deref(), "Hey welcome to club 69").unwrap();

    loop {

        let n = stream.deref().read(&mut buffer).map_err(|err| {
            let _ = message_sender.send(Message::ClientDisconnected{author: stream.clone()});
            eprintln!("ERR: could not read from the client, {err}")
        });

        let _ = message_sender.send(Message::NewMessage{author: stream.clone(),
        bytes: buffer[0..n.unwrap()].to_vec()}).map_err(|err| {
            eprintln!("ERR: could not send message to server thread, {err}")
        });

    }

}


fn main() {

    let address = "0.0.0.0:6969";
    
    let (message_sender, message_receiver) = mpsc::channel::<Message>();

    let listener = TcpListener::bind(&address).map_err(|err| {
        eprintln!("ERR: could not bind to address {address}, {err}");
    }).unwrap();

    println!("INFO: connected to {address}");

    thread::spawn(|| server(message_receiver));

    for stream in listener.incoming() {

        match stream {

            Ok(stream) => {

                let message_sender_clone = message_sender.clone();
                thread::spawn(move || {
                    handle_client(Arc::new(stream), message_sender_clone);
                });

            }

            Err(err) => {
                eprintln!("ERR: could not listen the stream, {err}");
            }

        }

    }

}
