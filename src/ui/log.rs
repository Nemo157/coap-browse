use std::sync::Arc;

use tokio_coap::message::Message as CoapMessage;
use tokio_coap::error::Error as CoapError;

use iced::{Element, Text, Command};

use super::{response::{self, Response}, error::Error};

#[derive(Debug)]
pub enum SessionLog {
    Request {
        url: String,
    },
    Response(Response),
    Error(Error),
}

#[derive(Copy, Clone, Debug)]
pub enum Msg {
    Response(response::Msg),
}

impl SessionLog {
    pub fn response(request: String, response: Result<Arc<CoapMessage>, Arc<CoapError>>) -> Self {
        match response {
            Ok(response) => SessionLog::Response(Response::new(request, response)),
            Err(error) => SessionLog::Error(Error::new(request, error)),
        }
    }

    pub fn update(&mut self, msg: Msg) -> Command<Msg> {
        match (self, msg) {
            (SessionLog::Response(response), Msg::Response(msg)) => response.update(msg).map(Msg::Response),
            _ => unreachable!(),
        }
    }

    pub fn view(&mut self) -> Element<'_, Msg> {
        match self {
            SessionLog::Request { url }
                => Text::new(format!("Request to {}", url)).into(),
            SessionLog::Response(response)
                => response.view().map(Msg::Response),
            SessionLog::Error(response)
                => response.view().map(|m| match m{}),
        }
    }
}
