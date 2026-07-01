use iced::{
    Element, Length, Task,
    widget::{
        Button, button, center, center_x, column, container, mouse_area, opaque, row, scrollable,
        stack, text,
    },
};
use iced_plot::{
    Color, HoverPickEvent, LineStyle, MarkerStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder,
    Series,
};
use rand::prelude::*;
use std::{
    collections::{HashMap, hash_map},
    io::{BufRead, Read, Seek, Write},
    rc::Rc,
};

use crate::log_reader::{LogInfo, MessageFormat};

mod log_reader;
mod params_table;

const MSGS_COLUMN_WIDTH_PX: f32 = 200.0;

#[derive(Debug, Clone)]
enum Widget {
    Plot,
    Table,
}

struct ArdupilotLogParserApp {
    log_info: LogInfo,
    plot_widget: PlotWidget,
    table_widget: params_table::Table,
    show_modal: bool,
    current_widget: Widget,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFileDialog,
    ShowMsgPlot(u8),
    PlotMsg(PlotUiMessage),
    TableMsg(params_table::Message),
    ShowModal(bool),
    SwitchWidget(Widget),
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
        // let positions = (0..100)
        //     .map(|i| {
        //         let x = i as f64 * 0.1;
        //         let y = (x * 0.5).sin();
        //         [x, y]
        //     })
        //     .collect();

        // let s1 = Series::line_only(positions, LineStyle::solid().with_pixel_width(4.0))
        //     .with_label("sine_line_only")
        //     .with_color(Color::from_rgb(0.3, 0.3, 0.9));

        // let positions = (0..50)
        //     .map(|i| {
        //         let x = i as f64 * 0.2;
        //         let y = (x * 0.3).cos() + 0.5;
        //         [x, y]
        //     })
        //     .collect();
        // let s2 = Series::markers_only(positions, MarkerStyle::circle(6.0))
        //     .with_label("cosine_markers_only")
        //     .with_color(Color::from_rgb(0.9, 0.3, 0.3));

        // let positions = (0..30)
        //     .map(|i| {
        //         let x = i as f64 * 0.3;
        //         let y = (x * 0.8).sin() - 0.5;
        //         [x, y]
        //     })
        //     .collect();
        // let s3 = Series::new(positions, MarkerStyle::square(4.0), LineStyle::dashed(10.0))
        //     .with_label("both_markers_and_lines")
        //     .with_color(Color::from_rgb(0.3, 0.9, 0.3));

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
            // .add_series(s1)
            // .add_series(s2)
            // .add_series(s3)
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

        let table_widget = params_table::Table::new();

        (
            Self {
                log_info: Default::default(),
                plot_widget,
                table_widget,
                show_modal: false,
                current_widget: Widget::Plot,
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
                        Ok(log_info) => {
                            self.log_info = log_info;
                            // let json = serde_json::to_string(&msgs).unwrap();
                            // let mut json_file = std::fs::File::create("out.json").unwrap();
                            // json_file.write_all(json.as_bytes());
                            self.table_widget.update(params_table::Message::ShowParamsInfo(
                                self.log_info.params.clone(),
                            ));
                        }
                        Err(err_msg) => {
                            eprintln!("{err_msg}");
                        }
                    }
                }
            }
            Message::ShowMsgPlot(msg_id) => {
                let msg_format_maybe = self.log_info.formats.get(&msg_id);
                let msg_values_maybe = self.log_info.msgs.get(&msg_id);
                if let (Some(msg_format), Some(msg_values)) = (msg_format_maybe, msg_values_maybe) {
                    // println!("{:#?}", msg_format);
                    // println!("{:#?}", msg_values);

                    let time_field_index =
                        msg_format.labels.iter().position(|label| label == "TimeUS");

                    self.plot_widget
                        .series_ids()
                        .iter()
                        .for_each(|shape_id| self.plot_widget.remove_series(shape_id).unwrap());
                    for (field_index, field_name) in msg_format.labels.iter().enumerate() {
                        let r = rand::random_range(0.0..1.0);
                        let g = rand::random_range(0.0..1.0);
                        let b = rand::random_range(0.0..1.0);

                        let series = Series::line_only(
                            msg_values
                                .iter()
                                .map(|fields| {
                                    [
                                        time_field_index.unwrap_or(0) as f64,
                                        fields[field_index].to_f64().unwrap_or(0.0),
                                    ]
                                })
                                .collect(),
                            LineStyle::solid().with_pixel_width(4.0),
                        )
                        .with_label(format!("{} {}", msg_format.name, field_name))
                        .with_color(Color::from_rgb(r, g, b));
                        match self.plot_widget.add_series(series) {
                            Ok(_) => {}
                            Err(msg) => {
                                eprintln!("{:#?}", msg);
                            }
                        }
                    }
                }
            }
            Message::PlotMsg(plot_msg) => {
                self.plot_widget.update(plot_msg);
            }
            Message::TableMsg(table_msg) => {
                self.table_widget.update(table_msg);
            }
            Message::ShowModal(show) => {
                self.show_modal = show;
            }
            Message::SwitchWidget(widget) => {
                self.current_widget = widget;
            }
        }
    }

    fn view(&'_ self) -> Element<'_, Message> {
        let column = {
                let mut sorted_msg_formats =
                    self.log_info.formats.iter().collect::<Vec<(&u8, &MessageFormat)>>();
                sorted_msg_formats.sort_by(|(_, msg_format1), (_, msg_format2)| {
                    msg_format1.name.cmp(&msg_format2.name)
                });
                column(
                    sorted_msg_formats
                        .iter()
                        .map(|(msg_id, msg_format)| {
                            button(text(msg_format.name.as_str())).on_press(
                                /*Message::ShowModal(true),*/
                                Message::ShowMsgPlot(**msg_id),
                            )
                        })
                        .map(Element::from),
                )
        };

        let signup = container(
            column![
                text("Sign Up").size(24),
                column![button(text("Submit")),].spacing(10)
            ]
            .spacing(20),
        )
        .width(300)
        .padding(10)
        .style(container::rounded_box);

        let content = row![
            scrollable(center_x(column![
                button("Plot").on_press(Message::SwitchWidget(Widget::Plot)),
                button("Table").on_press(Message::SwitchWidget(Widget::Table)),
                button("Choose file").on_press(Message::OpenFileDialog),
                column,
            ]))
            .width(Length::Fixed(MSGS_COLUMN_WIDTH_PX)),
            container(match self.current_widget {
                Widget::Plot => self.plot_widget.view().map(Message::PlotMsg),
                Widget::Table => self.table_widget.view().map(Message::TableMsg),
            })
            .width(Length::Fill),
        ];

        if self.show_modal {
            modal(content, signup, Message::ShowModal(false))
        } else {
            content.into()
        }
    }
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}
