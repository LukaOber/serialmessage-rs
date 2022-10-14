[![Version](https://img.shields.io/crates/v/serialmessage.svg)](https://crates.io/crates/serialmessage)

# serialmessage

`serialmessage` enables you to pack serial data into a fast, reliable,
and packetized form for communicating with e.g. a Microcontroller. It is compatible with
the [Arduino SerialTransfer][STArduino] and [Python pySerialTransfer][STPython]
libraries by [PowerBroker2][GithubPowerBroker]. This crate is designed to be used
with any serial crate you desire, and does therefore not implement any serial
communication on its own. This crate is optionally fully no_std compatible and can
be used on any Microcontroller of your choice.

The message format:
- uses start and stop bytes
- uses packet ids
- uses consistent overhead byte stuffing
- uses CRC-8 (Polynomial 0x9B with lookup table)
- allows the use of dynamically sized packets (packets can have payload lengths anywhere from 0 to 254 bytes)
- can transfer bytes, ints, floats, structs, arrays, vectors

# Packet Anatomy:
```text
01111110 00000000 11111111 00000000 00000000 00000000 ... 00000000 10000001
|      | |      | |      | |      | |      | |      | | | |      | |______|__Stop byte (constant)
|      | |      | |      | |      | |      | |      | | | |______|___________8-bit CRC
|      | |      | |      | |      | |      | |      | |_|____________________Rest of payload
|      | |      | |      | |      | |      | |______|________________________2nd payload byte
|      | |      | |      | |      | |______|_________________________________1st payload byte
|      | |      | |      | |______|__________________________________________# of payload bytes
|      | |      | |______|___________________________________________________COBS Overhead byte
|      | |______|____________________________________________________________Packet ID
|______|_____________________________________________________________________Start byte (constant)

```

# Example
## Basic Example

```rust
use serialmessage::{SerMsg, ParseState};

let send_data_vec: Vec<u8> = vec![1, 2, 3, 4];
let send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
// Send the message bytes with a serial crate of your choice

//Parsing received bytes
let mut ser_msg = SerMsg::new();
let (parse_state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);
match parse_state {
    ParseState::DataReady => {
        let rcvd_data = ser_msg.return_read_data();
        assert_eq!(&send_data_vec, rcvd_data);
    }
    _ => {
        println!("Parsestate: {:?}", parse_state);
    }
}
```
## Using the crate provided examples
If you flash your microcontroller with the code provided in the /examples/arduino_code/ folder you can try the provided examples yourself. 

```no_rust
cargo run --example echo your_port
```

# Status of this crate
## Current state

This crate is fully functional and tested, I use this crate a lot at work and had no issues so far. If you do encounter an issue or have a question just let me know.

## Goals for the future
- Add no_std microcontroller example
- Add a timeout parse error

# no_std usage
Disable the default features of this crate and you are good to go.


[GithubPowerBroker]: https://github.com/PowerBroker2
[STArduino]: https://github.com/PowerBroker2/SerialTransfer
[STPython]: https://github.com/PowerBroker2/pySerialTransfer