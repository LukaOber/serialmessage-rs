//! # serialmessage
//!
//! `serialmessage` enables you to pack serial data into a fast, reliable,
//! and packetized form for communicating with e.g. a Microcontroller. It is compatible with
//! the [Arduino SerialTransfer][STArduino] and [Python pySerialTransfer][STPython]
//! libraries by [PowerBroker2][GithubPowerBroker]. This crate is designed to be used
//! with any serial crate you desire, and does therefore not implement any serial
//! communication on its own. This crate is optionally fully no_std compatible and can
//! be used on any Microcontroller of your choice.
//!
//! The message format:
//! - uses start and stop bytes
//! - uses packet ids
//! - uses consistent overhead byte stuffing
//! - uses CRC-8 (Polynomial 0x9B with lookup table)
//! - allows the use of dynamically sized packets (packets can have payload lengths anywhere from 0 to 254 bytes)
//! - can transfer bytes, ints, floats, structs, arrays, vectors
//!
//! # Packet Anatomy:
//! ```text
//! 01111110 00000000 11111111 00000000 00000000 00000000 ... 00000000 10000001
//! |      | |      | |      | |      | |      | |      | | | |      | |______|__Stop byte (constant)
//! |      | |      | |      | |      | |      | |      | | | |______|___________8-bit CRC
//! |      | |      | |      | |      | |      | |      | |_|____________________Rest of payload
//! |      | |      | |      | |      | |      | |______|________________________2nd payload byte
//! |      | |      | |      | |      | |______|_________________________________1st payload byte
//! |      | |      | |      | |______|__________________________________________# of payload bytes
//! |      | |      | |______|___________________________________________________COBS Overhead byte
//! |      | |______|____________________________________________________________Packet ID
//! |______|_____________________________________________________________________Start byte (constant)
//!
//! ```
//!
//! # Examples
//! ### Basic Example
//!
//! ```rust
//! use serialmessage::{SerMsg, ParseState};
//!
//! let send_data_vec: Vec<u8> = vec![1, 2, 3, 4];
//! let send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
//! // Send the message bytes with a serial crate of your choice
//!
//! //Parsing received bytes
//! let mut ser_msg = SerMsg::new();
//! let (parse_state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);
//! match parse_state {
//!     ParseState::DataReady => {
//!         let rcvd_data = ser_msg.return_read_data();
//!         assert_eq!(&send_data_vec, rcvd_data);
//!     }
//!     _ => {
//!         println!("Parsestate: {:?}", parse_state);
//!     }
//! }
//! ```
//!
//! ### no_std zerocopy Example
//! ```rust
//! use serialmessage::{SerMsg, ParseState};
//! use zerocopy::{AsBytes, FromBytes};
//!
//! #[repr(C, packed)]
//! #[derive(FromBytes, AsBytes, Debug, Clone, Copy, PartialEq)]
//! struct ExampleData {
//!     i_32: i32,
//!     f_32: f32,
//! }
//!
//! let send_struct = ExampleData {i_32: 26, f_32: 55.845};
//! let send_bytes = send_struct.as_bytes();
//! let (send_msg, msg_len) = SerMsg::create_msg_arr(send_bytes, 1).unwrap();
//! // Send: send_msg[..msg_len];
//!
//! //Parsing received bytes
//! let mut ser_msg = SerMsg::new();
//! let (parse_state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg[..msg_len]);
//! match parse_state {
//!     ParseState::DataReady => {
//!         let rcvd_struct = ExampleData::read_from(ser_msg.return_read_data()).unwrap();
//!         assert_eq!(send_struct, rcvd_struct);
//!     }
//!     _ => {
//!         println!("Parsestate: {:?}", parse_state);
//!     }
//! }
//! ```
//!
//! [GithubPowerBroker]: https://github.com/PowerBroker2
//! [STArduino]: https://github.com/PowerBroker2/SerialTransfer
//! [STPython]: https://github.com/PowerBroker2/pySerialTransfer

#![no_std]

/// Shows the progress/error when parsing bytes with [SerMsg.parse_read_bytes()][parse_read_bytes].
///
/// # Example
///
/// ```rust
/// use serialmessage::{SerMsg, ParseState};
/// let rcvd_message_vec: Vec<u8> = vec![126, 1, 0, 4, 0, 72, 105, 33, 246, 129];
/// let mut ser_msg = SerMsg::new();
/// let (parse_state, _parsed_bytes) = ser_msg.parse_read_bytes(&rcvd_message_vec);
/// match parse_state {
///     ParseState::DataReady => {
///         // Do something with the received data
///     }
///     // No error occurred, but no complete message is ready
///     ParseState::Continue => (),
///     // Handle or ignore these error
///     ParseState::CrcError => (),
///     ParseState::HighPayloadError => (),
///     ParseState::StopByteError => (),
///     ParseState::COBSError => (),
/// }
/// ```
///
/// [parse_read_bytes]: SerMsg::parse_read_bytes()
#[derive(Debug)]
pub enum ParseState {
    /// The bytes were handled successfully, but no complete message is ready
    Continue,
    /// The complete message was parsed successfully and the data is ready
    DataReady,
    /// The CrcCheck failed, the message is most likely corrupted
    CrcError,
    /// The Payload length exceeded its maximum of 254
    HighPayloadError,
    /// The StopByte was not found after the CrcCheck
    StopByteError,
    /// Couldn't successfully unpack the Consistent Overhead Byte Stuffing
    COBSError,
}

enum FindByte {
    Start,
    Id,
    Overhead,
    PayloadLen,
    Payload,
    Crc,
    End,
}

/// Struct that implements all the functionality of this crate
pub struct SerMsg {
    msg_state: FindByte,
    payload_len: u8,
    cobs_byte: u8,
    rcvd_id: u8,
    rcvd_data: [u8; 254],
    rcvd_ind: usize,
}

impl Default for SerMsg {
    fn default() -> Self {
        Self::new()
    }
}

impl SerMsg {
    const START_BYTE: u8 = 126;
    const STOP_BYTE: u8 = 129;
    const MAX_PACKET_SIZE: u8 = 254;

    const LOOKUP_TABLE: [u8; 256] = [
        0x00, 0x9b, 0xad, 0x36, 0xc1, 0x5a, 0x6c, 0xf7, 0x19, 0x82, 0xb4, 0x2f, 0xd8, 0x43, 0x75,
        0xee, 0x32, 0xa9, 0x9f, 0x04, 0xf3, 0x68, 0x5e, 0xc5, 0x2b, 0xb0, 0x86, 0x1d, 0xea, 0x71,
        0x47, 0xdc, 0x64, 0xff, 0xc9, 0x52, 0xa5, 0x3e, 0x08, 0x93, 0x7d, 0xe6, 0xd0, 0x4b, 0xbc,
        0x27, 0x11, 0x8a, 0x56, 0xcd, 0xfb, 0x60, 0x97, 0x0c, 0x3a, 0xa1, 0x4f, 0xd4, 0xe2, 0x79,
        0x8e, 0x15, 0x23, 0xb8, 0xc8, 0x53, 0x65, 0xfe, 0x09, 0x92, 0xa4, 0x3f, 0xd1, 0x4a, 0x7c,
        0xe7, 0x10, 0x8b, 0xbd, 0x26, 0xfa, 0x61, 0x57, 0xcc, 0x3b, 0xa0, 0x96, 0x0d, 0xe3, 0x78,
        0x4e, 0xd5, 0x22, 0xb9, 0x8f, 0x14, 0xac, 0x37, 0x01, 0x9a, 0x6d, 0xf6, 0xc0, 0x5b, 0xb5,
        0x2e, 0x18, 0x83, 0x74, 0xef, 0xd9, 0x42, 0x9e, 0x05, 0x33, 0xa8, 0x5f, 0xc4, 0xf2, 0x69,
        0x87, 0x1c, 0x2a, 0xb1, 0x46, 0xdd, 0xeb, 0x70, 0x0b, 0x90, 0xa6, 0x3d, 0xca, 0x51, 0x67,
        0xfc, 0x12, 0x89, 0xbf, 0x24, 0xd3, 0x48, 0x7e, 0xe5, 0x39, 0xa2, 0x94, 0x0f, 0xf8, 0x63,
        0x55, 0xce, 0x20, 0xbb, 0x8d, 0x16, 0xe1, 0x7a, 0x4c, 0xd7, 0x6f, 0xf4, 0xc2, 0x59, 0xae,
        0x35, 0x03, 0x98, 0x76, 0xed, 0xdb, 0x40, 0xb7, 0x2c, 0x1a, 0x81, 0x5d, 0xc6, 0xf0, 0x6b,
        0x9c, 0x07, 0x31, 0xaa, 0x44, 0xdf, 0xe9, 0x72, 0x85, 0x1e, 0x28, 0xb3, 0xc3, 0x58, 0x6e,
        0xf5, 0x02, 0x99, 0xaf, 0x34, 0xda, 0x41, 0x77, 0xec, 0x1b, 0x80, 0xb6, 0x2d, 0xf1, 0x6a,
        0x5c, 0xc7, 0x30, 0xab, 0x9d, 0x06, 0xe8, 0x73, 0x45, 0xde, 0x29, 0xb2, 0x84, 0x1f, 0xa7,
        0x3c, 0x0a, 0x91, 0x66, 0xfd, 0xcb, 0x50, 0xbe, 0x25, 0x13, 0x88, 0x7f, 0xe4, 0xd2, 0x49,
        0x95, 0x0e, 0x38, 0xa3, 0x54, 0xcf, 0xf9, 0x62, 0x8c, 0x17, 0x21, 0xba, 0x4d, 0xd6, 0xe0,
        0x7b,
    ];

    /// Returns a new SerMsg instance for parsing read data
    pub fn new() -> SerMsg {
        let msg_state = FindByte::Start;
        let rcvd_id: u8 = 0;
        let cobs_byte: u8 = 0;
        let payload_len: u8 = 0;
        let rcvd_data: [u8; 254] = [0; 254];
        let rcvd_ind: usize = 0;

        SerMsg {
            msg_state,
            rcvd_id,
            cobs_byte,
            payload_len,
            rcvd_data,
            rcvd_ind,
        }
    }

    /// Returns the data of the parsed message, should only be used after the [ParseState][ParseState] is [DataReady][DataReady]
    ///
    /// [ParseState]: ParseState
    /// [DataReady]: ParseState::DataReady
    pub fn return_read_data(&self) -> &[u8] {
        &self.rcvd_data[..self.rcvd_ind]
    }

    /// Returns the id of the parsed message, should only be used after the [ParseState][ParseState] is [DataReady][DataReady]
    ///
    /// [ParseState]: ParseState
    /// [DataReady]: ParseState::DataReady
    pub fn return_msg_id(&self) -> u8 {
        self.rcvd_id
    }

    /// no_std function to create a message.
    /// Packs the slice into the message format and returns an array with a fixed length
    /// of the maximum message size and the last index of the relevant bytes in the array.
    /// Returns None if the input slice exceeds the maximum supported length of 254 bytes.
    pub fn create_msg_arr(data: &[u8], id: u8) -> Option<([u8; 260], usize)> {
        if data.len() > SerMsg::MAX_PACKET_SIZE as usize {
            return None;
        }
        let mut msg: [u8; 260] = [0; 260];
        msg[0] = SerMsg::START_BYTE;
        msg[1] = id;
        msg[3] = data.len() as u8;
        for (i, &val) in data.iter().enumerate() {
            msg[i + 4] = val;
        }
        msg[2] = SerMsg::pack_cobs(&mut msg[4..4 + data.len()]);
        msg[4 + data.len()] = SerMsg::retrieve_crc(&msg[4..4 + data.len()]);
        msg[5 + data.len()] = SerMsg::STOP_BYTE;
        Some((msg, data.len() + 6))
    }

    /// Parses the bytes of the input slice. Returns a [ParseState][ParseState] and the amount of bytes parsed
    /// when an error occured, a complete message was parsed or when all bytes were read.
    ///
    /// [ParseState]: ParseState
    pub fn parse_read_bytes(&mut self, arr: &[u8]) -> (ParseState, usize) {
        for (i, val) in arr.iter().enumerate() {
            let state = self.parse_byte(*val);
            match state {
                ParseState::Continue => (),
                _ => return (state, i + 1),
            }
        }
        (ParseState::Continue, arr.len())
    }

    fn retrieve_crc(data_slice: &[u8]) -> u8 {
        let mut calc_crc = 0;
        for val in data_slice.iter() {
            calc_crc = SerMsg::LOOKUP_TABLE[(calc_crc ^ val) as usize];
        }
        calc_crc
    }

    fn unpack_cobs(&mut self) -> bool {
        if self.cobs_byte <= SerMsg::MAX_PACKET_SIZE {
            if (self.cobs_byte as usize) >= self.payload_len as usize {
                return false;
            }
            while self.rcvd_data[self.cobs_byte as usize] > 0 {
                let delta: u8 = self.rcvd_data[self.cobs_byte as usize];

                // check if delta make us point outside payload region
                // saturating_add max out at 255. And it works, because:
                // self.payload_len<=SerMsg::MAX_PACKET_SIZE<255
                if (delta.saturating_add(self.cobs_byte)) >= self.payload_len {
                    return false;
                }
                self.rcvd_data[self.cobs_byte as usize] = SerMsg::START_BYTE;
                self.cobs_byte += delta;
            }
            self.rcvd_data[self.cobs_byte as usize] = SerMsg::START_BYTE;
            true
        } else {
            true
        }
    }

    fn pack_cobs(data_slice: &mut [u8]) -> u8 {
        let mut overhead_byte = 0xFF;
        for (f_ind, val) in (0_u8..).zip(data_slice.iter()) {
            if *val == SerMsg::START_BYTE {
                overhead_byte = f_ind;
                break;
            }
        }
        if (data_slice.len() <= SerMsg::MAX_PACKET_SIZE as usize) && (overhead_byte < 0xFF) {
            let mut last_start_byte = 0;

            let mut r_ind = data_slice.len();
            for _ in 0..data_slice.len() {
                r_ind -= 1;
                if data_slice[r_ind] == SerMsg::START_BYTE {
                    last_start_byte = r_ind;
                    break;
                }
            }
            r_ind = data_slice.len() - 1;
            for _ in 0..data_slice.len() {
                if data_slice[r_ind] == SerMsg::START_BYTE {
                    data_slice[r_ind] = (last_start_byte - r_ind) as u8;
                    last_start_byte = r_ind;
                }
                if r_ind == 0 {
                    break;
                }
                r_ind -= 1;
            }
        }
        overhead_byte
    }

    fn parse_byte(&mut self, val: u8) -> ParseState {
        match self.msg_state {
            FindByte::Start => {
                if val == SerMsg::START_BYTE {
                    self.rcvd_data.fill(0);
                    self.rcvd_ind = 0;
                    self.msg_state = FindByte::Id;
                }
                ParseState::Continue
            }

            FindByte::Id => {
                self.rcvd_id = val;
                self.msg_state = FindByte::Overhead;
                ParseState::Continue
            }

            FindByte::Overhead => {
                self.cobs_byte = val;
                self.msg_state = FindByte::PayloadLen;
                ParseState::Continue
            }

            FindByte::PayloadLen => {
                self.payload_len = val;
                if val > SerMsg::MAX_PACKET_SIZE {
                    self.msg_state = FindByte::Start;
                    ParseState::HighPayloadError
                } else if val == 0 {
                    self.msg_state = FindByte::Crc;
                    ParseState::Continue
                } else {
                    self.msg_state = FindByte::Payload;
                    ParseState::Continue
                }
            }

            FindByte::Payload => {
                self.rcvd_data[self.rcvd_ind] = val;
                self.rcvd_ind += 1;
                if (self.payload_len as usize - self.rcvd_ind) == 0 {
                    self.msg_state = FindByte::Crc;
                }
                ParseState::Continue
            }

            FindByte::Crc => {
                if val == SerMsg::retrieve_crc(&self.rcvd_data[..self.rcvd_ind]) {
                    self.msg_state = FindByte::End;
                    ParseState::Continue
                } else {
                    self.msg_state = FindByte::Start;
                    ParseState::CrcError
                }
            }

            FindByte::End => {
                self.msg_state = FindByte::Start;
                if val == SerMsg::STOP_BYTE {
                    if self.unpack_cobs() {
                        ParseState::DataReady
                    } else {
                        ParseState::COBSError
                    }
                } else {
                    ParseState::StopByteError
                }
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature="alloc")] {
        extern crate alloc;
        use alloc::{vec::Vec};
        impl SerMsg {
            /// Packs the slice into the message format and returns a Vector containing the message bytes.
            /// Returns None if the input slice exceeds the maximum supported length of 254 bytes.
            #[allow(clippy::vec_init_then_push)]
            pub fn create_msg_vec(data: &[u8], id: u8) -> Option<Vec<u8>> {
                if data.len() > SerMsg::MAX_PACKET_SIZE as usize {
                    return None;
                }
                let mut data_vec: Vec<u8> = Vec::with_capacity(data.len() + 6);
                data_vec.push(SerMsg::START_BYTE);
                data_vec.push(id);
                data_vec.push(0); // 0 as a placeholder for the CRC
                data_vec.push(data.len() as u8);
                data_vec.extend(data);
                data_vec[2] = SerMsg::pack_cobs(&mut data_vec[4..4 + data.len()]);
                let crc = SerMsg::retrieve_crc(&data_vec[4..4 + data.len()]);
                data_vec.push(crc);
                data_vec.push(SerMsg::STOP_BYTE);
                Some(data_vec)
            }
        }
    }
}
