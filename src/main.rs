use iced::{
    Element, Task,
    widget::{button, column},
};
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
    // messages: HashMap<u8, Vec<Vec<FormatType>>>,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFileDialog,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        panic!("Please specify filename argument");
    }
    let log_file_path = &args[1];

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
    fn new() -> (Self, Task<Message>) {
        (Self {}, Task::none())
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::OpenFileDialog => {
                let choosen_file = rfd::FileDialog::new()
                    .add_filter("bin", &["bin, BIN"])
                    .pick_file();
                if let Some(file) = choosen_file {}
            }
        }
    }

    fn view(&'_ self) -> Element<'_, Message> {
        column![button("Choose file").on_press(Message::OpenFileDialog)]
            .padding(20)
            .into()
    }
}
