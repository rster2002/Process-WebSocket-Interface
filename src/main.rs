use std::io::{Read, Write};
use std::io::ErrorKind::WouldBlock;
use std::process::{Command, Stdio};
use std::io::Result;
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use std::sync::mpsc::channel;
use tungstenite::{accept, WebSocket};
use tungstenite::Message;
use crossbeam_channel::{bounded, Sender, Receiver};

fn main() -> () {
    let (out_sender, out_receiver) = bounded(4096);
    let (in_sender, in_receiver): (Sender<String>, Receiver<String>) = bounded(4096);

    spawn(move || {
        let mut output = Command::new("node")
            .arg("test.js")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to attach");

        let mut stdout = output.stdout.take().expect("Failed to take stdout.");
        let stdin = output.stdin.as_mut().expect("Failed to open stdin");

        let mut s = [0u8; 1];
        loop {
            let mut current_line: Vec<u8> = vec!();

            loop {
                let in_message = in_receiver.try_recv();

                if let Ok(command) = in_message {
                    println!("Writing to stdin");
                    let bytes = command.as_bytes();
                    stdin.write_all(bytes).expect("Could not write to stdin");
                }

                stdout.read(&mut s).expect("Cannot read");
                let current_byte = s[0];
                current_line.push(current_byte);

                if current_byte == 10 {
                    break;
                }
            }

            // print!("{}", String::from_utf8(current_line).unwrap());
            out_sender.send(current_line).expect("Unable to send");
        }
    });

    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    for stream in server.incoming() {
        let socket_out_receiver = out_receiver.clone();
        let socket_in_sender = in_sender.clone();

        let mut parent_stream = stream.expect("Cannot open stream");
        parent_stream.set_nonblocking(true).expect("Could not set to non-blocking");

        spawn (move || {
            let mut websocket = accept(parent_stream).expect("Failed to start");

            loop {
                let local_stream = websocket.get_mut();

                let mut read_buffer = [0u8; 1];
                let peek_result = local_stream.peek(&mut read_buffer);

                if let Ok(_) = peek_result {
                    let in_message = websocket.read_message().expect("Unable to read message");

                    if in_message.is_close() {
                        println!("Closing");
                        break;
                    }

                    if in_message.is_text() {
                        let text_message = in_message.into_text().expect("Not text");
                        socket_in_sender.send(text_message).expect("Could not send");
                    }
                }

                let string = socket_out_receiver.recv().expect("Unable to receive");

                let bytes = string.to_owned();
                let message = String::from_utf8(bytes).expect("Unable to format");
                websocket.write_message(Message::text(message)).expect("Unable to send");
            }
        });
    }
}
