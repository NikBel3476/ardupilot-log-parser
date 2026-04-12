fn read_log() {
    let log_file = std::fs::File::open(log_file_path)
        .unwrap_or_else(|_| panic!("Cannot open file {log_file_path}"));
    let mut reader = std::io::BufReader::new(log_file);

    // let mut buf = vec![];
    // let _ = reader.read_to_end(&mut buf).expect("Failed to read file");

    // let mut read_state = ReadState::AwaitHead;
    // for byte in buf {
    //     read_state = match read_state {
    //         ReadState::AwaitHead => {
    //             if byte == HEAD1 || byte == HEAD2 {
    //                 println!("head found");
    //                 ReadState::AwaitAttribute
    //             } else {
    //                 ReadState::AwaitHead
    //             }
    //         },
    //         ReadState::AwaitAttribute => {
    //             if byte == FMT_ATTRIBUTE {
    //                 ReadState::ParseMessage
    //             } else {
    //                 ReadState::AwaitHead
    //             }
    //         },
    //         ReadState::ParseMessage => {

    //         }
    //     }
    // }

    let mut head = [0u8; 3];
    let mut msg_formats = HashMap::new();
    let mut messages = HashMap::new();

    let mut msg_id_param: Option<u8> = None;
    let mut msg_id_unit: Option<u8> = None;
    let mut msg_id_format_unit: Option<u8> = None;
    let mut msg_id_mult: Option<u8> = None;

    while reader.read_exact(&mut head).is_ok() {
        let [head1, head2, msg_id] = head;
        if head1 != HEAD1 || head2 != HEAD2 {
            // println!("Bad header: {:02X} {:02X}", head1, head2);
            reader.seek_relative(-(head.len() - 1) as i64).unwrap();
            continue;
        }

        if msg_id == MSG_ID_FMT {
            let mut msg_header = [0u8; 2];
            if reader.read_exact(&mut msg_header).is_err() {
                break;
            }
            let [msg_id_fmt, msg_len] = msg_header;
            println!("Found fmt for {msg_id_fmt}");
            match msg_formats.entry(msg_id_fmt) {
                hash_map::Entry::Vacant(fmt_entry) => {
                    // e.insert(msg_len);
                    const NAME_OFFSET: usize = 0;
                    const NAME_LEN: usize = 4;
                    const FORMAT_OFFSET: usize = NAME_LEN;
                    const FORMAT_LEN: usize = 16;
                    const LABELS_OFFSET: usize = NAME_LEN + FORMAT_LEN;
                    const LABELS_LEN: usize = 64;
                    let mut fmt_msg_body = [0; NAME_LEN + FORMAT_LEN + LABELS_LEN];
                    if reader.read_exact(&mut fmt_msg_body).is_err() {
                        break;
                    }
                    let name = &fmt_msg_body[NAME_OFFSET..(NAME_OFFSET + NAME_LEN)];
                    let format = &fmt_msg_body[FORMAT_OFFSET..(FORMAT_OFFSET + FORMAT_LEN)];
                    let labels = &fmt_msg_body[LABELS_OFFSET..(LABELS_OFFSET + LABELS_LEN)];
                    let mut labels_list = std::str::from_utf8(labels)
                        .unwrap()
                        .split(',')
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>();
                    let last_index = labels_list.len() - 1;
                    labels_list[last_index] = labels_list
                        .last()
                        .unwrap()
                        .trim_end_matches('\0')
                        .to_string();
                    let name = std::str::from_utf8(name)
                        .unwrap()
                        .trim_end_matches('\0')
                        .to_string();
                    match name.as_str() {
                        "PARM" => {
                            msg_id_param = Some(msg_id_fmt);
                        }
                        "UNIT" => {
                            msg_id_unit = Some(msg_id_fmt);
                        }
                        "FMTU" => {
                            msg_id_format_unit = Some(msg_id_fmt);
                        }
                        "MULT" => {
                            msg_id_mult = Some(msg_id_fmt);
                        }
                        _ => {}
                    }
                    fmt_entry.insert(MessageFormat {
                        id: msg_id_fmt,
                        length: msg_len,
                        name,
                        format: std::str::from_utf8(format)
                            .unwrap()
                            .trim_end_matches('\0')
                            .chars()
                            .collect(),
                        labels: labels_list,
                    });
                    messages.insert(msg_id_fmt, Vec::new());
                }
                hash_map::Entry::Occupied(e) => {
                    println!(
                        "FMT for msg {msg_id_fmt} already met. msg len: {}, len from msg: {msg_len}",
                        e.get().length
                    );
                }
            }
            continue;
        } else if let (
            hash_map::Entry::Occupied(fmt_entry),
            hash_map::Entry::Occupied(mut msg_entry),
        ) = (msg_formats.entry(msg_id), messages.entry(msg_id))
        {
            // let mut msg_body = vec![0u8; e.get().length.into()];
            // reader.read_exact(&mut msg_body).unwrap();
            let mut msg_fields = vec![];
            for format_char in &fmt_entry.get().format {
                let field = match format_char {
                    'a' => {
                        let mut buf = [0u8; 2 * 32];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        unsafe {
                            FormatType::Int16Len32(Box::from((buf.align_to::<i16>().1).to_vec()))
                        }
                    }
                    'b' => {
                        let mut buf = [0u8; 1];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Int8(i8::from_be_bytes(buf))
                    }
                    'B' => {
                        let mut buf = [0u8; 1];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Uint8(u8::from_be_bytes(buf))
                    }
                    'h' => {
                        let mut buf = [0u8; 2];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Int16(i16::from_be_bytes(buf))
                    }
                    'H' => {
                        let mut buf = [0u8; 2];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Uint16(u16::from_be_bytes(buf))
                    }
                    'i' => {
                        let mut buf = [0u8; 4];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Int32(i32::from_be_bytes(buf))
                    }
                    'I' => {
                        let mut buf = [0u8; 4];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Uint32(u32::from_be_bytes(buf))
                    }
                    'f' => {
                        let mut buf = [0u8; 4];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Float(f32::from_be_bytes(buf))
                    }
                    'd' => {
                        let mut buf = [0u8; 8];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Double(f64::from_be_bytes(buf))
                    }
                    'n' => {
                        let mut buf = [0u8; 4];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::CharLen4(Box::from(buf))
                    }
                    'N' => {
                        let mut buf = [0u8; 16];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::CharLen16(Box::from(buf))
                    }
                    'Z' => {
                        let mut buf = [0u8; 64];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::CharLen64(Box::from(buf))
                    }
                    'c' => {
                        let mut buf = [0u8; 2];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Int16Mul100(i16::from_be_bytes(buf))
                    }
                    'C' => {
                        let mut buf = [0u8; 2];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Uint16Mul100(u16::from_be_bytes(buf))
                    }
                    'e' => {
                        let mut buf = [0u8; 4];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Int32Mul100(i32::from_be_bytes(buf))
                    }
                    'E' => {
                        let mut buf = [0u8; 4];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Uint32Mul100(u32::from_be_bytes(buf))
                    }
                    'L' => {
                        let mut buf = [0u8; 4];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Int32LatLon(i32::from_be_bytes(buf))
                    }
                    'M' => {
                        let mut buf = [0u8; 1];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Uint8FlightMode(u8::from_be_bytes(buf))
                    }
                    'q' => {
                        let mut buf = [0u8; 8];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Int64(i64::from_be_bytes(buf))
                    }
                    'Q' => {
                        let mut buf = [0u8; 8];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        FormatType::Uint64(u64::from_be_bytes(buf))
                    }
                    char => {
                        println!("{:#?}", fmt_entry);
                        todo!("Undefined format type: `{char}`");
                    }
                };
                msg_fields.push(field);
            }
            msg_entry.get_mut().push(msg_fields);
        }

        // MSG_ID_PARAM => {
        //     time_us + name + value + default_value
        //     let mut msg_header = [0u8; 8 + ];
        //     reader.read_exact(&mut msg_header).unwrap();
        // }

        // if msg_len < 6 {
        //     println!("Msg must be more than 5 bytes");
        //     continue
        // }
        // let mut msg_body = vec![0u8; (msg_len - 5).into()];
        // reader.read_exact(&mut msg_body).unwrap();
        // println!("Msg body: {:02X?}", msg_body);
        // messages.push(msg_body);

        // if let std::collections::hash_map::Entry::Vacant(e) = lengths.entry(msg_type) {

        // }

        // i += 1;
        // if i > 3 {
        //     return;
        // }
    }

    println!("{:#?}", msg_id_param);
    println!("{:#?}", msg_id_unit);
    println!("{:#?}", msg_id_format_unit);
    println!("{:#?}", msg_id_mult);

    // println!("{:#?}", messages.get(&34).unwrap());
}
