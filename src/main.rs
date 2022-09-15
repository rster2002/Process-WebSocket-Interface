use std::io::{Read, Write};
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
        // let socket_in_sender = in_sender.clone();

        let mut parent_stream = stream.expect("Cannot open stream");
        parent_stream.set_nonblocking(true).expect("Could not set to non-blocking");

        // let stdout_steam = parent_stream.try_clone().expect("Cannot clone stream");
        // let stdin_steam = parent_stream.try_clone().expect("Cannot clone stream");

        spawn (move || {
            let mut websocket = accept(parent_stream).expect("Failed to start");

            loop {
                let local_stream = websocket.get_mut();

                let mut read_buffer = [0u8; 1];
                let read_result = local_stream.read(&mut read_buffer);

                if let Ok(_) = read_result {
                    local_stream.flush().expect("n");
                    let in_message = websocket.read_message().expect("Unable to read message");

                    println!("m: {}", in_message.to_text().expect("Not text"));

                    // println!("m: {}", read_buffer[0] as char);
                    // local_stream.flush().expect("Could not flush");
                }

                // parent_stream.read(&mut read_buffer).expect("Unable to read");

                //

                // if let Ok(msg_content) = msg {
                //     println!("stdin: {}", msg_content.to_text().unwrap());
                // }

                let string = socket_out_receiver.recv().expect("Unable to receive");

                let bytes = string.to_owned();
                let message = String::from_utf8(bytes)
                    .expect("Unable to format");
                websocket.write_message(Message::text(message)).unwrap();

                // We do not want to send back ping/pong messages.
                // if msg.is_binary() || msg.is_text() {
                //     websocket.write_message(msg).unwrap();
                // }
            }
        });

        // spawn (move || {
        //     let mut websocket = accept(stdin_steam).expect("Could not accept stream");
        //
        //     loop {
        //         let msg = websocket.read_message().unwrap();
        //
        //         if msg.is_text() {
        //             let command = msg.into_text().expect("Not a string");
        //             socket_in_sender.send(command).expect("TODO: panic message");
        //         }
        //     }
        // });
    }


    // let mut output_stream = output.stdout.expect("STDOUT could not be opened");
    // loop {
    //     let mut output_string = String::new();
    //     let mut take = output_stream
    //
    //     println!("a: {}", output_string);
    // }

    // let status = output.wait().expect("Error at exit");
    // println!("End of the line {}", status);

    // let mut i = output.stdout.expect("Error");
    //
    // loop {
    //     // let string = String::from_utf8_lossy(&i)?;
    //     let mut std_output = String::new();
    //     i.read_to_string(&mut std_output)?;
    //
    //     println!("{}", std_output);
    //
    //
    // }
}
