use std::io::prelude::*;
use std::net::TcpStream;
use std::net::Shutdown;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::io::BufReader;
use std::fmt;

//##################################################
// Client
//##################################################

pub struct StompClient {
    stream : TcpStream,
    receipt_count : i64,
    send_receipts : bool,
}

impl StompClient {

    pub fn new(stream : TcpStream) -> StompClient {
        StompClient {
            stream        : stream,
            receipt_count : 0,
            send_receipts : false,
        }
    }

    pub fn connect(&mut self) -> io::Result<Frame> {
        let connect_frame = Frame {
            command  : Command::CONNECT,
            headers  : vec![Header::new("accept-version","1.1,1.2")],
            body     : Vec::new(),
        };

        try!(self.send_frame(&connect_frame));
        let response = self.read_frame();
        response
    }

    pub fn disconnect(mut self) -> io::Result<Frame> {
        let disconnect_frame = Frame {
            command  : Command::DISCONNECT,
            headers  : vec![Header::new("receipt", self.receipt_count.to_string().as_ref())],
            body     : Vec::new(),
        };
        try!(self.send_frame(&disconnect_frame));
        let frame = try!(self.read_frame());
        try!(self.stream.shutdown(Shutdown::Both));
        Ok(frame)
    }

    pub fn read_frame(&mut self) -> io::Result<Frame> {
        let mut reader = BufReader::new(&self.stream);
        let mut buff = String::new();

        loop {
            //Skip trailing whitespace in case they were not read in last message
            try!(reader.read_line(&mut buff));
            if !buff.trim().is_empty() {
                break;
            }
            buff.clear();
        }

        let command = Command::from_string(buff.trim().as_ref()).unwrap();
        buff.clear();
        //Parse headers and content-lenght
        let mut content_lenght_present = false;
        let mut content_lenght = 0 as usize;
        let mut headers = Vec::new();
        while let Ok(_) = reader.read_line(&mut buff) {
            { //Scope for the header
                let header = buff.trim();
                if header.is_empty() {
                    break; //End of headers
                }
                let key_and_value : Vec<&str> = header.split(':').collect();
                if key_and_value.len() != 2 {
                    return Err(Error::new(ErrorKind::InvalidData,
                               format!("Bad header: {}", header)));
                }

                let key = key_and_value[0];
                let value = key_and_value[1];
                if key == "content-length" {
                    content_lenght_present = true;
                    match value.parse::<usize>() {
                        Ok(lenght) => content_lenght = lenght,
                        _ => return Err(Error::new(ErrorKind::InvalidData,
                                        format!("Bad content_lenght: {}", header)))
                    }
                }
                headers.push(Header::new(key, value));
            }
            buff.clear();
        }

        if !content_lenght_present {
            return Err(Error::new(ErrorKind::InvalidData,
                "Required header content-length not present"));
        }

        let mut body : Vec<u8> = Vec::with_capacity(content_lenght);
        if content_lenght != 0 {
            let read_bytes = try!(reader.read_until(0, &mut body));
            if read_bytes - 1 != content_lenght {
                return Err(Error::new(ErrorKind::InvalidData,
                    format!("Body and content-length do not match. read:{}, content-length:{}",
                    read_bytes, content_lenght)));
            }
            let l = body.len();
            body.remove(l - 1); //Remove the 0 terminator
        }

        Ok(Frame {
            command : command,
            headers : headers,
            body    : body,
        })
    }

    pub fn send_frame(&mut self, frame : &Frame) -> Result<()> {
        let receipt = if self.send_receipts {
            self.receipt_count = self.receipt_count + 1;
            Some(self.receipt_count)
        } else {
            None
        };
        let msg = frame.serialize(receipt);
        try!(self.stream.write_all(msg.as_bytes()));
        try!(self.stream.flush());
        Ok(())
    }

    pub fn send_message(&mut self, destination : &str, body: &[u8]) -> Result<()>{
        let mut vec_body = Vec::with_capacity(body.len());
        vec_body.extend(body);
        let message_frame = Frame {
            command  : Command::SEND,
            headers  : vec![Header::new("content-type", "text/plain"),
                            Header::new("destination", destination)],
            body     : vec_body,
        };
        self.send_frame(&message_frame)
    }

    pub fn subscribe_to_queue(&mut self, id : &str, queue: &str) -> io::Result<()> {
        let subscribe_frame = Frame {
            command  : Command::SUBSCRIBE,
            headers  : vec![Header::new("id", id),
                            Header::new("destination", queue),
                            Header::new("ack", "auto")],
            body     : Vec::new(),
        };
        try!(self.send_frame(&subscribe_frame));
        Ok(())
    }
}

//##################################################
// Frames
//##################################################

pub struct Frame {
    pub command       : Command,
    pub headers       : Vec<Header>,
    pub body          : Vec<u8>,
}

impl fmt::Display for Frame {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Frame {

    pub fn to_string(&self) -> String {
        self.serialize(None)
    }

    pub fn serialize(&self, receipt : Option<i64>) -> String {
        let mut s = String::new();

        //Command
        s.push_str(format!("{}\r\n", self.command.to_string()).as_ref());

        //Headers
        let mut has_lenght = false;
        for header in &self.headers {
            if header.key == "content-length" {
                has_lenght = true;
            }
            s.push_str(format!("{}:{}\r\n", header.key, header.value).as_ref());
        }
        if !has_lenght {
            s.push_str(format!("content-length:{}\r\n", self.body.len()).as_ref());
        }

        if let Some(receipt_id) = receipt {
            s.push_str(format!("receipt:{}\r\n", receipt_id).as_ref());
        }

        s.push_str("\r\n");

        //Body
        s.push_str(format!("{}\0", String::from_utf8(self.body.clone()).unwrap()).as_ref());
        s
    }
}

//##################################################
// Headers
//##################################################

pub struct Header {
    key : String,
    value : String,
}

impl Header {
    fn new(key : &str, value : &str) -> Header {
        Header{key : key.to_string(), value : value.to_string()}
    }
}

//##################################################
// Commands
//##################################################

#[allow(dead_code)]
pub enum Command {
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

#[allow(dead_code)]
impl Command {
    fn to_string(&self) -> String {
        let s = match *self {
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

    fn from_string(string : &str) -> Result<Command> {
        let command = string.to_uppercase();
        match command.as_ref() {
            "SEND"        => Ok(Command::SEND),
            "SUBSCRIBE"   => Ok(Command::SUBSCRIBE),
            "UNSUBSCRIBE" => Ok(Command::UNSUBSCRIBE),
            "BEGIN"       => Ok(Command::BEGIN),
            "COMMIT"      => Ok(Command::COMMIT),
            "ABORT"       => Ok(Command::ABORT),
            "ACK"         => Ok(Command::ACK),
            "NACK"        => Ok(Command::NACK),
            "DISCONNECT"  => Ok(Command::DISCONNECT),
            "CONNECT"     => Ok(Command::CONNECT),
            "STOMP"       => Ok(Command::STOMP),
            //Server
            "CONNECTED"   => Ok(Command::CONNECTED),
            "MESSAGE"     => Ok(Command::MESSAGE),
            "RECEIPT"     => Ok(Command::RECEIPT),
            "ERROR"       => Ok(Command::ERROR),
            _             =>
            Err(Error::new(ErrorKind::InvalidData, format!("Unknown command: {} ", string))),
        }
    }
}
