use iced::Element;
use iced_plot::{
    Color, HoverPickEvent, LineStyle, MarkerStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder,
    Series,
};
use std::{
    collections::{HashMap, hash_map},
    io::{BufRead, Read, Seek},
};

const HEAD1: u8 = 0xA3;
const HEAD2: u8 = 0x95;
const MSG_ID_FMT: u8 = 0x80;
const MSG_ID_PARAM: u8 = 0x20;

#[derive(Debug)]
enum FormatType {
    /// a
    Int16Len32(Box<Vec<i16>>),
    /// b
    Int8(i8),
    /// B
    Uint8(u8),
    /// h
    Int16(i16),
    /// H
    Uint16(u16),
    /// i
    Int32(i32),
    /// I
    Uint32(u32),
    /// f
    Float(f32),
    /// d
    Double(f64),
    /// n
    CharLen4(Box<[u8; 4]>),
    /// N
    CharLen16(Box<[u8; 16]>),
    /// Z
    CharLen64(Box<[u8; 64]>),
    /// **Legacy** c : int16_t * 100
    Int16Mul100(i16),
    /// **Legacy** C : uint16_t * 100
    Uint16Mul100(u16),
    /// **Legacy** e : int32_t * 100
    Int32Mul100(i32),
    /// **Legacy** E : uint32_t * 100
    Uint32Mul100(u32),
    /// L
    Int32LatLon(i32),
    /// M
    Uint8FlightMode(u8),
    /// q
    Int64(i64),
    /// Q
    Uint64(u64),
}

enum ReadState {
    AwaitHead,
    AwaitAttribute,
    ParseMessage,
}

#[derive(Debug)]
struct MessageFormat {
    id: u8,
    length: u8,
    name: String,
    format: Vec<char>,
    labels: Vec<String>,
}

// FMT msg
// Type: '128',
// length: '89',
// Name: 'FMT',
// Format: 'BBnNZ',
// Columns: ['Type', 'Length', 'Name' , 'Format', 'Columns']

struct ArdupilotLogParserApp {
    messages: HashMap<u8, Vec<Vec<FormatType>>>,
}

#[derive(Debug, Clone)]
enum Message {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        panic!("Please specify filename argument");
    }
    let log_file_path = &args[1];

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
            reader.seek_relative(-2).unwrap();
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

    // println!("{:#?}", msg_formats);
    // for (msg_id, fields) in &messages {
    //     if fields.is_empty() {
    //         println!("Msg {msg_id} not available");
    //     } else {
    //         println!("Msg {msg_id} available");
    //     }
    // }

    // for fields in messages.get(&MSG_ID_PARAM).unwrap() {
    //     // println!("Field: {:#?}", fields);
    //     for field in fields {
    //         if let FormatType::CharLen16(name) = field {
    //             println!(
    //                 "Param name: {}",
    //                 std::str::from_utf8(name.as_ref()).unwrap()
    //             );
    //         }
    //     }
    // }

    // let new = move || {
    //     let positions = (0..100)
    //         .map(|i| {
    //             let x = i as f64 * 0.1;
    //             let y = (x * 0.5).sin();
    //             [x, y]
    //         })
    //         .collect();

    //     let s1 = Series::line_only(positions, LineStyle::solid().with_pixel_width(4.0))
    //         .with_label("sine_line_only")
    //         .with_color(Color::from_rgb(0.3, 0.3, 0.9));

    //     let positions = (0..50)
    //         .map(|i| {
    //             let x = i as f64 * 0.2;
    //             let y = (x * 0.3).cos() + 0.5;
    //             [x, y]
    //         })
    //         .collect();
    //     let s2 = Series::markers_only(positions, MarkerStyle::circle(6.0))
    //         .with_label("cosine_markers_only")
    //         .with_color(Color::from_rgb(0.9, 0.3, 0.3));

    //     let positions = (0..30)
    //         .map(|i| {
    //             let x = i as f64 * 0.3;
    //             let y = (x * 0.8).sin() - 0.5;
    //             [x, y]
    //         })
    //         .collect();
    //     let s3 = Series::new(positions, MarkerStyle::square(4.0), LineStyle::dashed(10.0))
    //         .with_label("both_markers_and_lines")
    //         .with_color(Color::from_rgb(0.3, 0.9, 0.3));

    //     let positions = messages
    //         .get(&34)
    //         .unwrap()
    //         .iter()
    //         .enumerate()
    //         .map(|(i, msg_fields)| {
    //             if let FormatType::Uint64(msg_field) = msg_fields.first().unwrap() {
    //                 [i as f64, *msg_field as f64]
    //             } else {
    //                 [i as f64, 0f64]
    //             }
    //         })
    //         .collect();
    //     let s4 = Series::new(positions, MarkerStyle::circle(2.0), LineStyle::solid())
    //         .with_label("both_markers_and_lines")
    //         .with_color(Color::from_rgb(0.3, 0.9, 0.3));

    //     PlotWidgetBuilder::new()
    //         .with_hover_highlight_provider(|context, point| {
    //             if point.marker_style.is_none() {
    //                 point.marker_style = Some(MarkerStyle::circle(6.0));
    //             }
    //             Some(format!(
    //                 "Index: {}\nX: {:.2}\nY: {:.2}",
    //                 context.point_index, point.x, point.y
    //             ))
    //         })
    //         .with_pick_highlight_provider(|ctx, point| {
    //             if point.marker_style.is_none() {
    //                 // set plot 1 to star in pick highlight
    //                 point.marker_style = Some(MarkerStyle::triangle(6.0));
    //             }
    //             point.mask_padding = None;
    //             point.resize_marker(1.5);
    //             point.color = Color::from_rgb(1.0, 0.0, 0.0);
    //             Some(format!(
    //                 "Index: {}\nX: {:.2}\nY: {:.2}\n(Selected)",
    //                 ctx.point_index, point.x, point.y
    //             ))
    //         })
    //         .add_series(s1)
    //         .add_series(s2)
    //         .add_series(s3)
    //         .add_series(s4)
    //         .with_cursor_overlay(true)
    //         .with_cursor_provider(|x, y| format!("Your cursor is at: X: {x:.2}, Y: {y:.2}"))
    //         .with_y_label("Y Axis (Custom Font Size)")
    //         .with_x_label("X Axis (Custom Font Size)")
    //         .with_x_tick_formatter(|tick| format!("{:.1}s", tick.value))
    //         .with_tick_label_size(12.0)
    //         .with_axis_label_size(18.0)
    //         .with_crosshairs(true)
    //         .build()
    //         .unwrap()
    // };

    // println!("{:#?}", messages.get(&34).unwrap());

    iced::application(
        ArdupilotLogParserApp::new,
        ArdupilotLogParserApp::update,
        ArdupilotLogParserApp::view,
    )
    .run()
    .unwrap();

    Ok(())
}

impl ArdupilotLogParserApp {
    fn update(widget: &mut PlotWidget, message: PlotUiMessage) {
        let hover_pick_event = message.get_hover_pick_event();
        widget.update(message);
        // after PlotWidget's update, update the hover and pick points for the other series
        match hover_pick_event {
            Some(HoverPickEvent::Hover(point_id)) => {
                if let Some([x, _]) = widget.point_position(point_id) {
                    for series_id in widget.series_ids() {
                        if series_id != point_id.series_id
                            && let Some(p) = widget.nearest_point_horizontal(series_id, x)
                        {
                            widget.add_hover_point(p);
                        }
                    }
                }
            }
            Some(HoverPickEvent::Pick(point_id)) => {
                if let Some([x, _]) = widget.point_position(point_id) {
                    for series_id in widget.series_ids() {
                        if series_id != point_id.series_id
                            && let Some(p) = widget.nearest_point_horizontal(series_id, x)
                        {
                            widget.add_pick_point(p);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn view(widget: &PlotWidget) -> Element<'_, PlotUiMessage> {
        widget.view()
    }

    fn new() -> PlotWidget {
        let positions = (0..100)
            .map(|i| {
                let x = i as f64 * 0.1;
                let y = (x * 0.5).sin();
                [x, y]
            })
            .collect();

        let s1 = Series::line_only(positions, LineStyle::solid().with_pixel_width(4.0))
            .with_label("sine_line_only")
            .with_color(Color::from_rgb(0.3, 0.3, 0.9));

        let positions = (0..50)
            .map(|i| {
                let x = i as f64 * 0.2;
                let y = (x * 0.3).cos() + 0.5;
                [x, y]
            })
            .collect();
        let s2 = Series::markers_only(positions, MarkerStyle::circle(6.0))
            .with_label("cosine_markers_only")
            .with_color(Color::from_rgb(0.9, 0.3, 0.3));

        let positions = (0..30)
            .map(|i| {
                let x = i as f64 * 0.3;
                let y = (x * 0.8).sin() - 0.5;
                [x, y]
            })
            .collect();
        let s3 = Series::new(positions, MarkerStyle::square(4.0), LineStyle::dashed(10.0))
            .with_label("both_markers_and_lines")
            .with_color(Color::from_rgb(0.3, 0.9, 0.3));

        PlotWidgetBuilder::new()
            .with_hover_highlight_provider(|context, point| {
                if point.marker_style.is_none() {
                    point.marker_style = Some(MarkerStyle::circle(6.0));
                }
                Some(format!(
                    "Index: {}\nX: {:.2}\nY: {:.2}",
                    context.point_index, point.x, point.y
                ))
            })
            .with_pick_highlight_provider(|ctx, point| {
                if point.marker_style.is_none() {
                    // set plot 1 to star in pick highlight
                    point.marker_style = Some(MarkerStyle::triangle(6.0));
                }
                point.mask_padding = None;
                point.resize_marker(1.5);
                point.color = Color::from_rgb(1.0, 0.0, 0.0);
                Some(format!(
                    "Index: {}\nX: {:.2}\nY: {:.2}\n(Selected)",
                    ctx.point_index, point.x, point.y
                ))
            })
            .add_series(s1)
            .add_series(s2)
            .add_series(s3)
            .with_cursor_overlay(true)
            .with_cursor_provider(|x, y| format!("Your cursor is at: X: {x:.2}, Y: {y:.2}"))
            .with_y_label("Y Axis (Custom Font Size)")
            .with_x_label("X Axis (Custom Font Size)")
            .with_x_tick_formatter(|tick| format!("{:.1}s", tick.value))
            .with_tick_label_size(12.0)
            .with_axis_label_size(18.0)
            .with_crosshairs(true)
            .build()
            .unwrap()
    }
}
