use serialmessage::*;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn one_data_byte() {
        let send_data_vec: Vec<u8> = vec![1];
        let send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        let (send_msg_arr, msg_len) = SerMsg::create_msg_arr(&send_data_vec, 1).unwrap();
        if send_msg != send_msg_arr[..msg_len] {
            panic!("std and no_std functions produce different messages");
        }

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            ParseState::DataReady => {
                assert_eq!(&send_data_vec, ser_msg.return_read_data());
            }
            _ => {
                panic!();
            }
        }
    }

    #[test]
    fn multiple_data_byte() {
        let send_data_vec: Vec<u8> = vec![1, 2, 3];
        let send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        let (send_msg_arr, msg_len) = SerMsg::create_msg_arr(&send_data_vec, 1).unwrap();
        if send_msg != send_msg_arr[..msg_len] {
            panic!("std and no_std functions produce different messages");
        }

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            ParseState::DataReady => {
                assert_eq!(&send_data_vec, ser_msg.return_read_data());
            }
            _ => {
                panic!();
            }
        }
    }
    #[test]
    fn maximum_data_bytes() {
        let send_data_vec: Vec<u8> = vec![0; 254];
        let send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        let (send_msg_arr, msg_len) = SerMsg::create_msg_arr(&send_data_vec, 1).unwrap();
        if send_msg != send_msg_arr[..msg_len] {
            panic!("std and no_std functions produce different messages");
        }

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            ParseState::DataReady => {
                assert_eq!(&send_data_vec, ser_msg.return_read_data());
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn check_msg_id() {
        let id = 1;
        let send_data_vec: Vec<u8> = vec![1];
        let send_msg = SerMsg::create_msg_vec(&send_data_vec, id).unwrap();
        let (send_msg_arr, msg_len) = SerMsg::create_msg_arr(&send_data_vec, id).unwrap();
        if send_msg != send_msg_arr[..msg_len] {
            panic!("std and no_std functions produce different messages");
        }

        let mut ser_msg = SerMsg::new();
        let (_state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        if id == ser_msg.return_msg_id() {
            // Success
        } else {
            panic!()
        }
    }

    #[test]
    fn cobs_in_msg() {
        let send_data_vec: Vec<u8> = vec![126];
        let send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        let (send_msg_arr, msg_len) = SerMsg::create_msg_arr(&send_data_vec, 1).unwrap();
        if send_msg != send_msg_arr[..msg_len] {
            panic!("std and no_std functions produce different messages");
        }

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            ParseState::DataReady => {
                assert_eq!(&send_data_vec, ser_msg.return_read_data());
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn multiple_cobs_in_msg() {
        let send_data_vec: Vec<u8> = vec![126, 126, 126];
        let send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        let (send_msg_arr, msg_len) = SerMsg::create_msg_arr(&send_data_vec, 1).unwrap();
        if send_msg != send_msg_arr[..msg_len] {
            panic!("std and no_std functions produce different messages");
        }

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            ParseState::DataReady => {
                assert_eq!(&send_data_vec, ser_msg.return_read_data());
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn no_data_bytes() {
        let send_data_vec: Vec<u8> = vec![];
        let send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        let (send_msg_arr, msg_len) = SerMsg::create_msg_arr(&send_data_vec, 1).unwrap();
        if send_msg != send_msg_arr[..msg_len] {
            panic!("std and no_std functions produce different messages");
        }

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            ParseState::DataReady => {
                assert_eq!(&send_data_vec, ser_msg.return_read_data());
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn receive_multiple_messages() {
        let mut rcvd_int: Vec<u8> = Vec::new();
        let send_data_vec: Vec<u8> = vec![1];
        let mut send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        let second_send_data_vec: Vec<u8> = vec![2];
        let mut second_send_msg = SerMsg::create_msg_vec(&second_send_data_vec, 1).unwrap();
        let third_send_data_vec: Vec<u8> = vec![3];
        let mut third_send_msg = SerMsg::create_msg_vec(&third_send_data_vec, 1).unwrap();

        send_msg.append(&mut second_send_msg);
        send_msg.append(&mut third_send_msg);

        let mut ser_msg = SerMsg::new();
        let received_bytes = send_msg.len();
        let mut all_parsed_bytes = 0;

        while received_bytes > all_parsed_bytes {
            let (state, parsed_bytes) = ser_msg.parse_read_bytes(&send_msg[all_parsed_bytes..]);
            all_parsed_bytes += parsed_bytes;

            match state {
                ParseState::DataReady => {
                    rcvd_int.push(ser_msg.return_read_data()[0]);
                }
                _ => {
                    panic!()
                }
            }
        }
        assert_eq!(rcvd_int, vec![1, 2, 3]);
    }

    #[test]
    fn partial_messages() {
        let send_data_vec: Vec<u8> = vec![1];
        let send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg[..3]);

        match state {
            ParseState::Continue => {}
            _ => {
                panic!()
            }
        }

        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg[3..]);

        match state {
            ParseState::DataReady => {
                assert_eq!(&send_data_vec, ser_msg.return_read_data());
            }
            _ => {
                panic!()
            }
        }
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn over_max_payload_error() {
        let send_data_vec: Vec<u8> = vec![1];
        let mut send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        send_msg[3] = 255;

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            ParseState::HighPayloadError => {
                // Success
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn low_payload_error() {
        let send_data_vec: Vec<u8> = vec![1, 2];
        let mut send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        send_msg[3] = 1;

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            // Will show as CrcError as the last PayloadByte gets checked as the CrcByte
            ParseState::CrcError => {
                // Success
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn small_too_high_payload_error() {
        let send_data_vec: Vec<u8> = vec![1];
        let mut send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        send_msg[3] = 2;

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            // Will show as CrcError as the StopByte will get checked as the CrcByte
            ParseState::CrcError => {
                // Success
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn big_too_high_payload_error() {
        let send_data_vec: Vec<u8> = vec![1];
        let mut send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        send_msg[3] = 3;

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            // No Error as the CrcByte and StopByte get treated as data bytes
            ParseState::Continue => {
                // Success
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn wrong_crc_error() {
        let send_data_vec: Vec<u8> = vec![1];
        let mut send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        send_msg[5] = 0;

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            ParseState::CrcError => {
                // Success
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn wrong_stopbyte_error() {
        let send_data_vec: Vec<u8> = vec![1];
        let mut send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        send_msg[6] = 0;

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);

        match state {
            ParseState::StopByteError => {
                // Success
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn wrong_cobs_error() {
        let send_data_vec: Vec<u8> = vec![126, 1];
        let mut send_msg = SerMsg::create_msg_vec(&send_data_vec, 1).unwrap();
        send_msg[2] = 1;

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);
        match state {
            ParseState::COBSError => {
                // Success
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn exceeded_max_length() {
        let send_data_vec: Vec<u8> = vec![0; 255];
        let send_msg_vec_option = SerMsg::create_msg_arr(&send_data_vec, 1);
        let send_msg_arr_option = SerMsg::create_msg_arr(&send_data_vec, 1);

        if send_msg_vec_option.is_none() && send_msg_arr_option.is_none() {
            // Success
        } else {
            panic!()
        }
    }
}

mod zerocopy_tests {

    use super::*;
    use zerocopy::{AsBytes, FromBytes};
    // Using packed on both sides of the communication is recommended to lower
    // the size of the transmission and remove padding on all platforms.
    #[repr(C, packed)]
    #[derive(FromBytes, AsBytes, Debug, Clone, Copy, Default, PartialEq)]
    struct ExampleData {
        u_8: u8,
        i_8: i8,
        u_16: u16,
        i_16: i16,
        i_32: i32,
        u_32: u32,
        f_32: f32,
        u8_arr: [u8; 6],
    }

    #[test]
    fn example_test() {
        let send_struct = ExampleData::default();
        let send_bytes = send_struct.as_bytes();
        let send_msg = SerMsg::create_msg_vec(send_bytes, 1).unwrap();

        let mut ser_msg = SerMsg::new();
        let (state, _parsed_bytes) = ser_msg.parse_read_bytes(&send_msg);
        match state {
            ParseState::DataReady => {
                assert_eq!(send_bytes, ser_msg.return_read_data());
                let rcvd_struct = ExampleData::read_from(ser_msg.return_read_data()).unwrap();
                if rcvd_struct != send_struct {
                    panic!()
                }
            }
            _ => {
                panic!()
            }
        }
    }
}
