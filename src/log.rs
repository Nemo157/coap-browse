use vdom_rsjs::{VNode, VTag, VProperty};

use tokio_coap::message::Message as CoapMessage;
use tokio_coap::error::Error as CoapError;

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

impl SessionLog {
    pub fn render(&self) -> VNode<!> {
        match self {
            SessionLog::Request { url } => {
                VNode::Text(format!("Request to {}", url))
            }
            SessionLog::Response { request, response } => {
                VNode::Text(format!("Response for {}: {:?}", request, response))
            }
        }
    }
}
