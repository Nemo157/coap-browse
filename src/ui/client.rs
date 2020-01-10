use std::sync::Arc;

use tokio_coap::Client;
use tokio_coap::message::Message as CoapMessage;
use tokio_coap::error::Error as CoapError;
use serde_derive::{Deserialize, Serialize};
use iced::{Column, Row, Command, Element, widget::{text_input, scrollable}, TextInput, Scrollable};
use futures::compat::Future01CompatExt;

use super::log::{self, SessionLog};

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionTag {
    SubmitUrl,
}

#[derive(Debug, Clone)]
pub enum StateMessage {
    SubmitUrl,
    UrlChange(String),
    Response {
        request: String,
        response: Result<Arc<CoapMessage>, Arc<CoapError>>,
    },
    LogMsg {
        index: usize,
        msg: log::Msg,
    },
}

#[derive(Debug)]
pub struct State {
    session_log: Vec<SessionLog>,
    url: String,
    url_state: text_input::State,
    rt: tokio_compat::runtime::Runtime,
    scrollable_state: scrollable::State,
}

impl Default for State {
    fn default() -> Self {
        Self {
            session_log: Vec::new(),
            url: "".to_owned(),
            url_state: text_input::State::new(),
            rt: tokio_compat::runtime::Runtime::new().unwrap(),
            scrollable_state: scrollable::State::new(),
        }
    }
}

impl State {
    pub fn update(&mut self, msg: StateMessage) -> Command<StateMessage> {
        match msg {
            StateMessage::UrlChange(url) => {
                self.url = url;
                Command::none()
            }
            StateMessage::SubmitUrl => {
                self.session_log.push(SessionLog::Request { url: self.url.clone() });
                Command::perform({
                    let url = self.url.clone();
                    self.rt.spawn_handle_std(async move {
                        let client = Client::get(&url)?;
                        Ok(Arc::new(client.send().compat().await?))
                    })
                }, {
                    let url = self.url.clone();
                    move |response| StateMessage::Response { request: url.clone(), response: response.unwrap() }
                })
            }
            StateMessage::Response { request, response } => {
                self.session_log.push(SessionLog::response(request, response));
                Command::none()
            }
            StateMessage::LogMsg { index, msg } => {
                // TODO: This index will be out of date if the list changes
                self.session_log[index].update(msg).map(move |msg| StateMessage::LogMsg { index, msg })
            }
        }
    }

    pub fn view(&mut self) -> Element<'_, StateMessage> {
        Column::new()
                .push(Row::new().push(TextInput::new(&mut self.url_state, "coap url", &self.url, StateMessage::UrlChange).on_submit(StateMessage::SubmitUrl)))
                .push({
                    let mut logs = Scrollable::new(&mut self.scrollable_state);
                    for (index, log) in self.session_log.iter_mut().enumerate().rev() {
                        logs = logs.push(log.view().map(move |msg| StateMessage::LogMsg { index, msg }));
                    }
                    logs
                })
                .into()
    }
}
