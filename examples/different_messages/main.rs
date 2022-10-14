extern crate serialmessage;
extern crate serialport;

use serialmessage::{ParseState, SerMsg};
use std::{env, thread, time::Duration};
use zerocopy::{AsBytes, FromBytes};

#[repr(C, packed)]
#[derive(FromBytes, AsBytes, Debug, Clone, Copy)]
struct OneNumber {
    num: i32,
}

#[repr(C, packed)]
#[derive(FromBytes, AsBytes, Debug, Clone, Copy)]
struct TwoNumbers {
    num1: i16,
    num2: i16,
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
        Err(_) => {
            println!("No valid port was provided or there was an error accessing the port");
            print_port_names();
            panic!()
        }
    };

    let send_data = TwoNumbers { num1: 26, num2: 8 };

    let send_bytes = send_data.as_bytes();
    let send_msg = SerMsg::create_msg_vec(send_bytes, 1).unwrap();

    let _written_amount = port.write(&send_msg).unwrap();

    thread::sleep(Duration::from_millis(100));

    let mut ser_msg = SerMsg::new();
    let mut buffer = [0; 256];
    let read_amount = port.read(&mut buffer).unwrap();
    let read_bytes = buffer[..read_amount].to_owned();
    let mut all_parsed_bytes = 0;

    while read_amount > all_parsed_bytes {
        let (state, parsed_bytes) = ser_msg.parse_read_bytes(&read_bytes[all_parsed_bytes..]);
        all_parsed_bytes += parsed_bytes;

        match state {
            ParseState::DataReady => {
                if ser_msg.return_msg_id() == 1 {
                    let rcvd_two_nums = TwoNumbers::read_from(ser_msg.return_read_data()).unwrap();
                    println!("Rcvd: {:?}", rcvd_two_nums);
                } else if ser_msg.return_msg_id() == 2 {
                    let rcvd_one_num = OneNumber::read_from(ser_msg.return_read_data()).unwrap();
                    println!("Rcvd: {:?}", rcvd_one_num);
                } else {
                    println!("Invalid ID: {}", ser_msg.return_msg_id());
                }
            }
            _ => {
                println!("State: {:?}", state);
            }
        }
    }
}
