use std::{str, sync::Arc};

use tokio_coap::message::Message as CoapMessage;
use tokio_coap::error::Error as CoapError;
use tokio_coap::message::option::ContentFormat;
use tokio_coap::message::option::Option as O;

use iced::{Element, Text, Column};

use serde_json;
use serde_cbor;
use serde_cbor_diag;
use serde_xml;

#[derive(Debug)]
pub enum SessionLog {
    Request {
        url: String,
    },
    Response {
        request: String,
        response: Result<Arc<CoapMessage>, Arc<CoapError>>,
    },
}

fn render_raw_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(format!("{:#?}", payload))
        .into()
}

fn render_json_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(serde_json::from_slice::<serde_json::Value>(payload)
            .and_then(|p| serde_json::to_string_pretty(&p))
            .unwrap_or_else(|e| format!("{:?}", e)))
        .into()
}

fn render_cbor_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(serde_cbor::from_slice::<serde_cbor::Value>(payload)
            .map_err(|e| format!("{:?}", e))
            .and_then(|p| serde_cbor_diag::to_string_pretty(&p)
                .map_err(|e| format!("{:?}", e)))
            .unwrap_or_else(|e| e))
        .into()
}

fn render_xml_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(str::from_utf8(payload)
            .map_err(Box::<dyn ::std::error::Error>::from)
            .and_then(|p| serde_xml::from_str::<serde_xml::value::Element>(p)
                .map_err(Box::<dyn ::std::error::Error>::from))
            .map(|p| format!("{:#?}", p))
            .unwrap_or_else(|e| format!("{:#?}", e)))
        .into()
}

fn render_link_format_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(String::from_utf8_lossy(payload).into_owned())
        .into()
}

fn render_plain_text_payload(payload: &[u8]) -> Element<'static, !> {
    Text::new(String::from_utf8_lossy(payload).into_owned())
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

fn render_good_response(url: &str, msg: &CoapMessage) -> Element<'static, !> {
    let fmt = match msg.options.get::<ContentFormat>() {
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
        .push(Text::new(format!("Response for {}", url)))
        .push(Column::new()
            .push({
                Text::new(match (&fmt, fmt_name) {
                    (_, Some(fmt)) => format!("content format: {}", fmt),
                    (Some(fmt), _) => format!("content format: {:?}", fmt),
                    (None, _) => "unspecified content format".to_owned(),
                })
            })
            .push(render_payload(fmt, &msg.payload)))
        .push(Column::new()
            .push(Text::new("Raw message"))
            .push(Text::new(format!("{:#?}", msg))))
        .into()
}

fn render_bad_response(url: &str, err: &CoapError) -> Element<'static, !> {
    Column::new()
        .push(Text::new(format!("Error requesting {}", url)))
        .push(Text::new(format!("{:#?}", err)))
        .into()
}

fn render_response(url: &str, response: &Result<Arc<CoapMessage>, Arc<CoapError>>) -> Element<'static, !> {
    match response {
        Ok(msg) => render_good_response(url, msg).into(),
        Err(err) => render_bad_response(url, err).into(),
    }
}

impl SessionLog {
    pub fn view(&mut self) -> Element<'_, !> {
        match self {
            SessionLog::Request { url }
                => Text::new(format!("Request to {}", url)).into(),
            SessionLog::Response { request, response }
                => render_response(request, response),
        }
    }
}
