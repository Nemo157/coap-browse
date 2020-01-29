use std::{str, sync::Arc};

use tokio_coap::message::Message as CoapMessage;
use tokio_coap::message::option::ContentFormat;
use tokio_coap::message::option::Option as O;

use iced::{Color, Element, Text, Column, Button, widget::button, Row, Command};

use serde_json;
use serde_cbor;
use serde_cbor_diag;
use serde_xml;

use super::resources::MONOSPACE;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum DisplayType {
    Rendered,
    Raw,
}

#[derive(Debug)]
pub struct Response {
    request: String,
    response: Arc<CoapMessage>,
    rendered_button_state: button::State,
    raw_button_state: button::State,
    display: DisplayType,
}

#[derive(Copy, Clone, Debug)]
pub enum Msg {
    SwitchDisplay(DisplayType),
}

fn render_raw_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(format!("{:#?}", payload))
        .font(MONOSPACE.clone())
        .into()
}

fn render_json_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(serde_json::from_slice::<serde_json::Value>(payload)
            .and_then(|p| serde_json::to_string_pretty(&p))
            .unwrap_or_else(|e| format!("{:?}", e)))
        .font(MONOSPACE.clone())
        .into()
}

fn render_cbor_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(serde_cbor::from_slice::<serde_cbor::Value>(payload)
            .map_err(|e| format!("{:?}", e))
            .and_then(|p| serde_cbor_diag::to_string_pretty(&p)
                .map_err(|e| format!("{:?}", e)))
            .unwrap_or_else(|e| e))
        .font(MONOSPACE.clone())
        .into()
}

fn render_xml_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(str::from_utf8(payload)
            .map_err(Box::<dyn ::std::error::Error>::from)
            .and_then(|p| serde_xml::from_str::<serde_xml::value::Element>(p)
                .map_err(Box::<dyn ::std::error::Error>::from))
            .map(|p| format!("{:#?}", p))
            .unwrap_or_else(|e| format!("{:#?}", e)))
        .font(MONOSPACE.clone())
        .into()
}

fn render_link_format_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(String::from_utf8_lossy(payload).into_owned())
        .font(MONOSPACE.clone())
        .into()
}

fn render_plain_text_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(String::from_utf8_lossy(payload).into_owned())
        .font(MONOSPACE.clone())
        .into()
}

fn render_payload(fmt: Option<ContentFormat>, payload: &[u8]) -> Element<'static, !> {
    Column::new()
        .push(Text::new("Payload: "))
        .push({
            if fmt == Some(ContentFormat::new(0)) {
                render_plain_text_payload(payload)
            } else if fmt == Some(ContentFormat::new(40)) {
                render_link_format_payload(payload)
            } else if fmt == Some(ContentFormat::new(41)) {
                render_xml_payload(payload)
            } else if fmt == Some(ContentFormat::new(50)) {
                render_json_payload(payload)
            } else if fmt == Some(ContentFormat::new(60)) {
                render_cbor_payload(payload)
            } else {
                render_raw_payload(payload)
            }
        })
        .into()
}

impl Response {
    pub fn new(request: String, response: Arc<CoapMessage>) -> Self {
        Self {
            request,
            response,
            rendered_button_state: button::State::new(),
            raw_button_state: button::State::new(),
            display: DisplayType::Rendered,
        }
    }

    pub fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::SwitchDisplay(display) => {
                self.display = display;
                Command::none()
            }
        }
    }

    pub fn view(&mut self) -> Element<'_, Msg> {
        let fmt = match self.response.options.get::<ContentFormat>() {
            Some(ref fmt) if fmt.len() == 1 => Some(fmt[0]),
            Some(_) => {
                println!("Invalid ContentFormat");
                None
            }
            None => None,
        };

        let fmt_name = if fmt == Some(ContentFormat::new(0)) {
            Some("text/plain; charset=utf-8")
        } else if fmt == Some(ContentFormat::new(40)) {
            Some("application/link-format")
        } else if fmt == Some(ContentFormat::new(41)) {
            Some("application/xml")
        } else if fmt == Some(ContentFormat::new(50)) {
            Some("application/json")
        } else if fmt == Some(ContentFormat::new(60)) {
            Some("application/cbor")
        } else {
            None
        };

        Column::new()
            .push(Text::new(format!("Response for {}", self.request)).color(Color::from_rgb8(0x85, 0x99, 0x00)))
            .push({
                Text::new(match (&fmt, fmt_name) {
                    (_, Some(fmt)) => format!("content format: {}", fmt),
                    (Some(fmt), _) => format!("content format: {:?}", fmt),
                    (None, _) => "unspecified content format".to_owned(),
                })
            })
            .push(Row::new()
                .push(Text::new("Body"))
                .push(Button::new(&mut self.rendered_button_state, Text::new("rendered").color(if self.display == DisplayType::Rendered { [0.0, 1.0, 0.0] } else { [0.0, 0.0, 0.0] })).on_press(Msg::SwitchDisplay(DisplayType::Rendered)))
                .push(Button::new(&mut self.raw_button_state, Text::new("raw").color(if self.display == DisplayType::Raw { [0.0, 1.0, 0.0] } else { [0.0, 0.0, 0.0] })).on_press(Msg::SwitchDisplay(DisplayType::Raw))))
                .push(Text::new(""))
            .push(match self.display {
                DisplayType::Rendered => render_payload(fmt, &self.response.payload),
                DisplayType::Raw => Text::new(format!("{:#?}", self.response)).font(MONOSPACE.clone()).into(),
            }.map(|m| match m{}))
            .into()
    }
}

