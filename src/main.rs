use iced::{
    Element, Length, Task,
    widget::{Button, button, center_x, column, container, row, scrollable, text},
};
use iced_plot::{
    Color, HoverPickEvent, LineStyle, MarkerStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder,
    Series,
};
use std::{
    collections::{HashMap, hash_map},
    io::{BufRead, Read, Seek},
    rc::Rc,
};

mod log_reader;

struct ArdupilotLogParserApp {
    messages: Option<HashMap<u8, Vec<Vec<log_reader::FormatType>>>>,
    plot_widget: PlotWidget,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFileDialog,
    ShowMsgPlot(u8),
    PlotMessage(PlotUiMessage),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    let mut messages = Rc::new(None);
    if args.len() > 2 {
        // panic!("Please specify filename argument");
        let log_file_path = &args[1];
        if let Ok(msgs) = log_reader::read_log(&std::path::Path::new(log_file_path)) {
            messages = Rc::new(Some(msgs));
        }
    }

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
    .run()?;

    Ok(())
}

impl ArdupilotLogParserApp {
    fn new(/*messages: Option<Box<HashMap<u8, Vec<Vec<log_reader::FormatType>>>>>*/)
     -> (Self, Task<Message>) {
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

        let plot_widget = PlotWidgetBuilder::new()
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
            .unwrap();

        (
            Self {
                messages: None,
                plot_widget,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::OpenFileDialog => {
                let choosen_file = rfd::FileDialog::new()
                    .add_filter("bin", &["bin", "BIN"])
                    .pick_file();
                if let Some(file) = choosen_file {
                    match log_reader::read_log(&file) {
                        Ok(msgs) => {
                            self.messages = Some(msgs);
                        }
                        Err(err_msg) => {
                            eprintln!("{err_msg}");
                        }
                    }
                }
            }
            Message::ShowMsgPlot(msg_id) => {
                if let Some(msgs) = &self.messages {
                    let m = &msgs[&msg_id];
                    let series = Series::line_only(
                        m.iter()
                            .enumerate()
                            .map(|(i, fields)| {
                                let num = match fields[0] {
                                    log_reader::FormatType::Uint8(n) => n,
                                    _ => 0,
                                };
                                [i as f64, f64::from(num)]
                            })
                            .collect(),
                        LineStyle::solid().with_pixel_width(4.0),
                    )
                    .with_label("sine_line_only")
                    .with_color(Color::from_rgb(0.3, 0.3, 0.9));
                    self.plot_widget.add_series(series).unwrap();
                }
            }
            Message::PlotMessage(plot_msg) => {
                self.plot_widget.update(plot_msg);
            }
        }
    }

    fn view(&'_ self) -> Element<'_, Message> {
        let mut column = match &self.messages {
            Some(msgs) => column(
                msgs.iter()
                    .map(|(msg_id, fmt_messages)| {
                        button(text(msg_id)).on_press(Message::ShowMsgPlot(*msg_id))
                    })
                    .map(Element::from),
            ),
            None => column![],
        };

        row![
            scrollable(center_x(
                column![
                    button("Choose file").on_press(Message::OpenFileDialog),
                    column,
                ]
                .padding(20),
            ))
            .width(Length::Fixed(200.0)),
            container(self.plot_widget.view().map(Message::PlotMessage)).width(Length::Fill),
        ]
        .into()
    }
}
