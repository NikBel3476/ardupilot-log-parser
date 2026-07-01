use iced::font;
use iced::time::{Duration, hours, minutes};
use iced::widget::{
    center_x, center_y, column, container, row, scrollable, slider, table, text, tooltip,
};
use iced::{Center, Element, Fill, Font, Right, Left, Theme};

use crate::log_reader::{FormatType, MessageFormat, Parameter};

pub struct Table {
    // events: Vec<Event>,
    padding: (f32, f32),
    separator: (f32, f32),
    params: Vec<Parameter>
}

#[derive(Debug, Clone)]
pub enum Message {
    ShowParamsInfo(Vec<Parameter>),
    PaddingChanged(f32, f32),
    SeparatorChanged(f32, f32),
}

impl Table {
    pub fn new() -> Self {
        Self {
            // events: Event::list(),
            padding: (10.0, 5.0),
            separator: (1.0, 1.0),
            params: vec![]
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::PaddingChanged(x, y) => self.padding = (x, y),
            Message::SeparatorChanged(x, y) => self.separator = (x, y),
            Message::ShowParamsInfo(params ) => self.params = params
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let bold = |header| {
            text(header).font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
        };

        let table = {
            let columns = [
                table::column(bold("Name"), |param: &Parameter| text(param.name.as_str())).align_x(Left).align_y(Center),
                table::column(bold("Value"), |param: &Parameter| text!("{}", param.value)).align_x(Left).align_y(Center),
                table::column(bold("Default value"), |param: &Parameter| text!("{}", param.default_value)).align_x(Left).align_y(Center)
            ];
            table(columns, &self.params)
                .padding_x(self.padding.0)
                .padding_y(self.padding.1)
                .separator_x(self.separator.0)
                .separator_y(self.separator.1)
        };

        let controls = {
            let labeled_slider =
                |label,
                 range: std::ops::RangeInclusive<f32>,
                 (x, y),
                 on_change: fn(f32, f32) -> Message| {
                    row![
                        text(label).font(Font::MONOSPACE).size(14).width(100),
                        tooltip(
                            slider(range.clone(), x, move |x| on_change(x, y)),
                            text!("{x:.0}px").font(Font::MONOSPACE).size(10),
                            tooltip::Position::Left
                        ),
                        tooltip(
                            slider(range, y, move |y| on_change(x, y)),
                            text!("{y:.0}px").font(Font::MONOSPACE).size(10),
                            tooltip::Position::Right
                        ),
                    ]
                    .spacing(10)
                    .align_y(Center)
                };

            column![
                labeled_slider("Padding", 0.0..=30.0, self.padding, Message::PaddingChanged),
                labeled_slider(
                    "Separator",
                    0.0..=5.0,
                    self.separator,
                    Message::SeparatorChanged
                )
            ]
            .spacing(10)
            .width(400)
        };

        column![
            center_y(scrollable(center_x(table)).spacing(10)).padding(10),
            center_x(controls).padding(10).style(container::dark)
        ]
        .into()
    }
}
