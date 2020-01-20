#![feature(never_type)]
#![feature(try_blocks)]

#![warn(rust_2018_idioms)]

use iced::{Application, Command, Element, Settings};

mod ui;
mod executor;

#[derive(Default)]
struct CoapBrowse {
    client: ui::client::State,
}

impl Application for CoapBrowse {
    type Message = ui::client::StateMessage;
    type Executor = executor::TokioCompat;

    fn new() -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("coap-browse")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        self.client.update(message)
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.client.view()
    }
}

fn main() {
    CoapBrowse::run(Settings::default())
}
