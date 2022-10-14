extern crate serialmessage;
extern crate serialport;

use serialmessage::{ParseState, SerMsg};
use std::{env, thread, time::Duration};
use zerocopy::{AsBytes, FromBytes};

#[repr(C, packed)]
#[derive(FromBytes, AsBytes, Debug, Copy, Clone)]
struct ExampleData {
    u_8: u8,
    i_8: i8,
    u_16: u16,
    i_16: i16,
    f_32: f32,
    u8_arr: [u8; 6],
}

impl Default for ExampleData {
    fn default() -> Self {
        ExampleData {
            u_8: 100,
            i_8: -100,
            u_16: 10000,
            i_16: -10000,
            f_32: 1.23,
            u8_arr: [74, 111, 118, 101, 32, 55],
        }
    }
}

fn print_port_names() {
    let portlist = serialport::available_ports().unwrap();
    println!("Available ports: ");
    for portinfo in portlist.iter() {
        println!("{:?}", portinfo.port_name.to_owned());
    }
}

fn main() {
    if env::args().count() < 2 {
        println!("No port was provided as an argument");
        print_port_names();
        panic!()
    }
    let args: Vec<String> = env::args().collect();
    let port = serialport::new(args[1].to_owned(), 115_200).open();

    let mut port = match port {
        Ok(port) => port,
        Err(e) => {
            println!("No valid port was provided or there was an error accessing the port");
            println!("Error: {:?}", e);
            print_port_names();
            panic!()
        }
    };

    let example_send_data: ExampleData = Default::default();

    let send_bytes = example_send_data.as_bytes();
    let send_msg = SerMsg::create_msg_vec(send_bytes, 0).unwrap();

    let _write_amount = port.write(&send_msg).unwrap();

    println!("Sending: {:?}", example_send_data);
    println!("Send message contents:");
    println!("Start byte: {}", send_msg[0]);
    println!("Packet ID: {}", send_msg[1]);
    println!("COBS byte: {}", send_msg[2]);
    println!("Payload length: {}", send_msg[3]);
    for i in 0..send_msg[3] {
        println!("Databyte #{}: {}", i + 1, send_msg[4 + i as usize]);
    }
    println!("CRC byte: {}", send_msg[4 + send_msg[3] as usize]);
    println!("Stop byte: {}", send_msg[5 + send_msg[3] as usize]);

    thread::sleep(Duration::from_millis(100));

    let mut ser_msg = SerMsg::new();
    let mut buffer = [0; 256];
    let read_amount = port.read(&mut buffer).unwrap();
    let read_bytes = buffer[..read_amount].to_owned();

    println!("Read {} bytes into buffer", read_amount);

    let (state, parsed_bytes) = ser_msg.parse_read_bytes(&read_bytes);

    println!("Parsed {} bytes from buffer", parsed_bytes);

    match state {
        ParseState::DataReady => {
            let read_data: &[u8] = ser_msg.return_read_data();
            let rcvd_struct = ExampleData::read_from(read_data).unwrap();
            println!("Succesfully read and reconstructed the message");
            println!("Received: {:?}", rcvd_struct);
        }
        ParseState::Continue => {
            println!("There are still bytes missing from the message");
        }
        _ => println!(
            "{:?} ocurred, look at the documentation for more info",
            state
        ),
    }
}
