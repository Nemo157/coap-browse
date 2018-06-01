use std::str;

use client::Action;
use vdom_rsjs::{VNode, VTag};
use vdom_rsjs::render::{Render, Cache};

use tokio_coap::message::Message as CoapMessage;
use tokio_coap::error::Error as CoapError;
use tokio_coap::message::option::ContentFormat;
use tokio_coap::message::option::Option as O;

use serde_json;
use serde_cbor;
use serde_xml;

#[derive(Debug)]
pub enum SessionLog {
    Request {
        url: String,
    },
    Response {
        request: String,
        response: Result<CoapMessage, CoapError>,
    },
}

fn render_request(url: &str) -> VNode<Action> {
    VTag::new("div")
        .prop("style", "display:flex;flex-direction:column;border:1px solid #93a1a1")
        .child(VTag::new("div")
            .prop("style", "color:#586e75;padding:5px")
            .child(format!("Request to {}", url)))
        .into()
}

fn render_raw_payload(payload: &[u8]) -> VNode<Action> {
    VTag::new("pre")
        .child(format!("{:#?}", payload))
        .into()
}

fn render_json_payload(payload: &[u8]) -> VNode<Action> {
    VTag::new("pre")
        .child(serde_json::from_slice::<serde_json::Value>(payload)
            .and_then(|p| serde_json::to_string_pretty(&p))
            .unwrap_or_else(|e| format!("{:?}", e)))
        .into()
}

fn render_cbor_payload(payload: &[u8]) -> VNode<Action> {
    VTag::new("pre")
        .child(format!("{:#?}", serde_cbor::from_slice::<serde_cbor::Value>(payload)))
        .into()
}

fn render_xml_payload(payload: &[u8]) -> VNode<Action> {
    VTag::new("pre")
        .child(str::from_utf8(payload)
            .map_err(|e| Box::<::std::error::Error>::from(e))
            .and_then(|p| serde_xml::from_str::<serde_xml::value::Element>(p)
                .map_err(|e| Box::<::std::error::Error>::from(e)))
            .map(|p| format!("{:#?}", p))
            .unwrap_or_else(|e| format!("{:#?}", e)))
        .into()
}

fn render_link_format_payload(payload: &[u8]) -> VNode<Action> {
    VTag::new("pre")
        .child(String::from_utf8_lossy(payload).into_owned())
        .into()
}

fn render_plain_text_payload(payload: &[u8]) -> VNode<Action> {
    VTag::new("blockquote")
        .child(String::from_utf8_lossy(payload).into_owned())
        .into()
}

fn render_payload(fmt: Option<ContentFormat>, payload: &[u8]) -> VNode<Action> {
    VTag::new("div")
        .child("Payload: ")
        .child({
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

fn render_good_response(url: &str, msg: &CoapMessage) -> VNode<Action> {
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

    VTag::new("div")
        .prop("style", "display:flex;flex-direction:column;border:1px solid #93a1a1")
        .child(VTag::new("div")
            .prop("style", "color:#859900;border-bottom:1px solid #93a1a1;padding:5px")
            .child(format!("Response for {}", url)))
        .child(VTag::new("div")
            .prop("style", "border-bottom:1px solid #93a1a1;padding:5px")
            .child({
                match (&fmt, fmt_name) {
                    (_, Some(fmt)) => format!("content format: {}", fmt),
                    (Some(fmt), _) => format!("content format: {:?}", fmt),
                    (None, _) => "unspecified content format".to_owned(),
                }
            })
            .child(render_payload(fmt, &msg.payload)))
        .child(VTag::new("details")
            .prop("style", "color:#93a1a1;padding:5px")
            .child(VTag::new("summary").child("Raw message"))
            .child(VTag::new("pre")
                .prop("style", "margin-left:10px")
                .child(format!("{:#?}", msg))))
        .into()
}

fn render_bad_response(url: &str, err: &CoapError) -> VNode<Action> {
    VTag::new("div")
        .prop("style", "display:flex;flex-direction:column;border:1px solid #93a1a1")
        .child(VTag::new("div")
            .prop("style", "color:#dc322f;border-bottom:1px solid #93a1a1;padding:5px")
            .child(format!("Error requesting {}", url)))
        .child(VTag::new("pre").child(format!("{:#?}", err)))
        .into()
}

fn render_response(url: &str, response: &Result<CoapMessage, CoapError>) -> VNode<Action> {
    match response {
        Ok(msg) => render_good_response(url, msg),
        Err(err) => render_bad_response(url, err),
    }
}

impl Render<Action> for SessionLog {
    fn render(&self, cache: &mut Cache<Action>) -> VNode<Action> {
        match self {
            SessionLog::Request { url }
                => render_request(url),
            SessionLog::Response { request, response }
                => render_response(request, response),
        }
    }
}
