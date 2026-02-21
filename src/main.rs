use std::io::{Read, Seek};

const HEAD1: u8 = 163;
const HEAD2: u8 = 149;
const FMT_ATTRIBUTE: u8 = 128;

enum ReadState {
    AwaitHead,
    AwaitAttribute,
    ParseMessage
}

struct Message {
    r#type: String,
    length: u16,
    name: String,
    format: String,
    columns: Vec<String>
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
    let _ = reader.read_to_end(&mut buf).expect("Failed to read file");

    let mut read_state = ReadState::AwaitHead;
    for byte in buf {
        read_state = match read_state {
            ReadState::AwaitHead => {
                if byte == HEAD1 || byte == HEAD2 {
                    println!("head found");
                    ReadState::AwaitAttribute
                } else {
                    ReadState::AwaitHead
                }
            },
            ReadState::AwaitAttribute => {
                if byte == FMT_ATTRIBUTE {
                    ReadState::ParseMessage
                } else {
                    ReadState::AwaitHead
                }
            },
            ReadState::ParseMessage => {

            }
        }
    }
}
