use std::io::prelude::*;
use std::net::TcpStream;

struct Frame {
    pub command : Command,
    pub headers : Vec<Header>,
    pub body    : Vec<u8>,
}

impl Frame {
    pub fn to_string(self) -> String {
        let mut s = String::new();

        //Command
        s.push_str(format!("{}\r\n", self.command.to_string()).as_ref());

        //Headers
        s.push_str(format!("content-length:{}\r\n", self.body.len()).as_ref());
        for header in self.headers {
            s.push_str(format!("{}:{}\r\n", header.key, header.value).as_ref());
        }
        s.push_str("\r\n");

        //Body
        s.push_str(format!("{}\0", String::from_utf8(self.body).unwrap()).as_ref());
        s
    }
}

struct Header {
    key : String,
    value : String,
}

#[allow(dead_code)]
enum Command {
    //Client
    SEND,
    SUBSCRIBE,
    UNSUBSCRIBE,
    BEGIN,
    COMMIT,
    ABORT,
    ACK,
    NACK,
    DISCONNECT,
    CONNECT,
    STOMP,
    //Server
    CONNECTED,
    MESSAGE,
    RECEIPT,
    ERROR,
}

impl Command {
    fn to_string(self) -> String {
        let s = match self {
            //Client
            Command::SEND        => "SEND",
            Command::SUBSCRIBE   => "SUBSCRIBE",
            Command::UNSUBSCRIBE => "UNSUBSCRIBE",
            Command::BEGIN       => "BEGIN",
            Command::COMMIT      => "COMMIT",
            Command::ABORT       => "ABORT",
            Command::ACK         => "ACK",
            Command::NACK        => "NACK",
            Command::DISCONNECT  => "DISCONNECT",
            Command::CONNECT     => "CONNECT",
            Command::STOMP       => "STOMP",
            //Server
            Command::CONNECTED   => "CONNECTED",
            Command::MESSAGE     => "MESSAGE",
            Command::RECEIPT     => "RECEIPT",
            Command::ERROR       => "ERROR",
            //_                  => panic!("Unknown command"),
        };
        s.to_string()
    }
}

fn main() {

    let frame = Frame {
        command : Command::SEND,
        headers : Vec::new(),
        body    : "Hello 会意字 / 會意字".to_string().into_bytes(),
    };

    let body = frame.to_string();
    println!("{:?}", body);
    // let mut stream = TcpStream::connect("127.0.0.1:61613").unwrap();
    let mut stream = TcpStream::connect("127.0.0.1:12321").unwrap();
    let _ = stream.write_all(body.as_bytes());
    let _ = stream.flush();
    let _ = stream.shutdown(std::net::Shutdown::Both);
}
