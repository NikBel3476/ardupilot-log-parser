use std::io::Read;

const HEAD1: u8 = 0xA3;
const HEAD2: u8 = 0x95;
const FMT_ATTRIBUTE: u8 = 128;

enum ReadState {
    Header(HeaderReadState),
    ParseMessage(MsgBodyReadState),
}

enum HeaderReadState {
    Head1,
    Head2,
    MsgType
}

enum MsgBodyReadState {
    FormatType(u8),
    Length(u8),
    Body(u8)
}

struct Message {
    msg_type: u8,
    length: u8,
    name: String,
    format: String,
    columns: Vec<String>,
}

// FMT msg
// Type: '128',
// length: '89',
// Name: 'FMT',
// Format: 'BBnNZ',
// Columns: ['Type', 'Length', 'Name' , 'Format', 'Columns']

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        panic!("Please specify filename argument");
    }
    let log_file_path = &args[1];

    let log_file = std::fs::File::open(log_file_path)
        .unwrap_or_else(|_| panic!("Cannot open file {log_file_path}"));
    let mut reader = std::io::BufReader::new(log_file);
    let mut buf = vec![];
    let total_bytes = reader.read_to_end(&mut buf).expect("Failed to read file");

    let format_msg = Message {
        msg_type: HEAD1.into(),
        length: 89,
        name: "FMT".to_string(),
        format: "BBnNZ".to_string(),
        columns: vec![
            "Type".to_string(),
            "Length".to_string(),
            "Name".to_string(),
            "Format".to_string(),
            "Columns".to_string(),
        ],
    };
    let mut fmt = std::collections::HashMap::new();
    fmt.insert(format_msg.msg_type, format_msg);
    let mut read_state = ReadState::Header(HeaderReadState::Head1);
    let mut msg_counter = 0u32;
    let mut skipped_msgs = 0u32;
    let mut offset = 0;
    while offset < total_bytes {
        let byte = buf[offset];
        (read_state, offset) = match read_state {
            ReadState::Header(header_read_state) => match header_read_state {
                HeaderReadState::Head1 => {
                    if byte == HEAD1 {
                        (ReadState::Header(HeaderReadState::Head2), offset + 1)
                    } else {
                        (ReadState::Header(HeaderReadState::Head1), offset + 1)
                    }
                },
                HeaderReadState::Head2 => {
                    if byte == HEAD2 {
                        msg_counter += 1;
                        (ReadState::Header(HeaderReadState::MsgType), offset + 1)
                    } else {
                        skipped_msgs += 1;
                        (ReadState::Header(HeaderReadState::Head1), offset + 1)
                    }
                },
                HeaderReadState::MsgType => {
                    (ReadState::ParseMessage(MsgBodyReadState::FormatType(byte)), offset + 1)
                }
            }
            ReadState::ParseMessage(msg_body_read_state) => {
                match msg_body_read_state {
                    MsgBodyReadState::FormatType(msg_type) => {
                        println!("Message type: {msg_type}");
                        (ReadState::ParseMessage(MsgBodyReadState::Length(byte)), offset + 1)
                    }
                    MsgBodyReadState::Length(format_type) => {
                        println!("Format type: {format_type}");
                        (ReadState::ParseMessage(MsgBodyReadState::Body(byte)), offset + 1)
                    }
                    MsgBodyReadState::Body(msg_length) => {
                        let msg_end_offset = offset + usize::from(msg_length);
                        println!("Length: {msg_length}");
                        if msg_end_offset < buf.len() {
                            let msg_body = &buf[offset..msg_end_offset];
                        }
                        (ReadState::Header(HeaderReadState::Head1), msg_end_offset + 1)
                    }
                }
            }
        }
    }

    println!("Msg count: {msg_counter}. Skipped: {skipped_msgs}")
}
