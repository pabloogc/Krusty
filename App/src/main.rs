mod lib;

use std::net::TcpStream;
use lib::*;
use std::thread;
use std::sync::mpsc::channel;

fn main() {

    let (tx, rx) = channel();
    let ip = "127.0.0.1:61613";
    let queue = "q";
    let count = 3000;

    let stream = TcpStream::connect(ip).unwrap();
    let mut client = StompClient::new(stream);
    client.connect().unwrap();
    client.subscribe_to_queue("0", queue).unwrap();

    let t2 = thread::spawn(move || {
        let stream = TcpStream::connect(ip).unwrap();
        let mut client2 = StompClient::new(stream);
        client2.connect().unwrap();

        for i in 0..count {
            let body = format!("Message {} of {}", i + 1, count).to_string().into_bytes();
            client2.send_message(queue, &body).unwrap();
            rx.recv().unwrap();
        }

        client2.disconnect().unwrap();
    });

    for i in 0..count {
        let frame = client.read_frame().unwrap();
        println!("----------\n{}\n----------", frame.serialize(None));
        tx.send(i).unwrap();
    }

    client.disconnect().unwrap();
    let _ = t2.join();
}
