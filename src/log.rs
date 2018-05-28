use std::collections::HashMap;

use vdom_rsjs::{VNode, VTag};

use tokio_coap::message::Message as CoapMessage;
use tokio_coap::error::Error as CoapError;
use tokio_coap::message::option::ContentFormat;
use tokio_coap::message::option::Option as O;

use serde_json;
use serde_cbor;

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

fn render_request(url: &str) -> VNode<!> {
    VNode::Text(format!("Request to {}", url))
}

fn render_raw_payload(payload: &[u8]) -> VNode<!> {
    VNode::Tag(VTag {
        name: "pre".to_owned(),
        properties: HashMap::new(),
        children: vec![
            VNode::Text(format!("{:#?}", payload)),
        ],
        key: None,
        namespace: None,
    })
}

fn render_json_payload(payload: &[u8]) -> VNode<!> {
    VNode::Tag(VTag {
        name: "pre".to_owned(),
        properties: HashMap::new(),
        children: vec![
            VNode::Text(serde_json::from_slice::<serde_json::Value>(payload).and_then(|p| serde_json::to_string_pretty(&p)).unwrap_or_else(|e| format!("{:?}", e))),
        ],
        key: None,
        namespace: None,
    })
}

fn render_cbor_payload(payload: &[u8]) -> VNode<!> {
    VNode::Tag(VTag {
        name: "pre".to_owned(),
        properties: HashMap::new(),
        children: vec![
            VNode::Text(format!("{:#?}", serde_cbor::from_slice::<serde_cbor::Value>(payload))),
        ],
        key: None,
        namespace: None,
    })
}

fn render_payload(fmt: Option<ContentFormat>, payload: &[u8]) -> VNode<!> {
    VNode::Tag(VTag {
        name: "div".to_owned(),
        properties: HashMap::new(),
        children: vec![
            VNode::Text("Payload: ".to_owned()),
            if fmt == Some(ContentFormat::new(0)) {
                VNode::Text(String::from_utf8_lossy(payload).into_owned())
            } else if fmt == Some(ContentFormat::new(50)) {
                render_json_payload(payload)
            } else if fmt == Some(ContentFormat::new(60)) {
                render_cbor_payload(payload)
            } else {
                render_raw_payload(payload)
            }
        ],
        key: None,
        namespace: None,
    })
}

fn render_good_response(url: &str, msg: &CoapMessage) -> VNode<!> {
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
    } else if fmt == Some(ContentFormat::new(50)) {
        Some("application/json")
    } else if fmt == Some(ContentFormat::new(60)) {
        Some("application/cbor")
    } else {
        None
    };

    VNode::Tag(VTag {
        name: "div".to_owned(),
        properties: HashMap::new(),
        children: vec![
            VNode::Text(format!("Response for {}", url)),
            match (&fmt, fmt_name) {
                (_, Some(fmt)) => VNode::Text(format!("content format: {}", fmt)),
                (Some(fmt), _) => VNode::Text(format!("content format: {:?}", fmt)),
                (None, _) => VNode::Text("unspecified content format".to_owned()),
            },
            render_payload(fmt, &msg.payload),
            VNode::Text("raw:".to_owned()),
            VNode::Tag(VTag {
                name: "pre".to_owned(),
                properties: HashMap::new(),
                children: vec![
                    VNode::Text(format!("{:#?}", msg)),
                ],
                key: None,
                namespace: None,
            })
        ],
        key: None,
        namespace: None,
    })
}

fn render_bad_response(url: &str, err: &CoapError) -> VNode<!> {
    VNode::Tag(VTag {
        name: "div".to_owned(),
        properties: HashMap::new(),
        children: vec![
            VNode::Text(format!("Error requesting {}", url)),
            VNode::Text(format!("{:?}", err)),
        ],
        key: None,
        namespace: None,
    })
}

fn render_response(url: &str, response: &Result<CoapMessage, CoapError>) -> VNode<!> {
    match response {
        Ok(msg) => render_good_response(url, msg),
        Err(err) => render_bad_response(url, err),
    }
}

impl SessionLog {
    pub fn render(&self) -> VNode<!> {
        match self {
            SessionLog::Request { url }
                => render_request(url),
            SessionLog::Response { request, response }
                => render_response(request, response),
        }
    }
}
