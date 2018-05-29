use std::borrow::Cow;
use std::collections::HashMap;

use vdom_rsjs::{VNode, VTag, VProperty};
use tokio_core::reactor::Handle;
use tokio_coap::Client;
use tokio_coap::message::Message as CoapMessage;
use tokio_coap::error::Error as CoapError;

use futures::{Sink, Stream, Future, future::{self, Either}};
use futures::unsync::mpsc;

use log::SessionLog;

type ShouldRender = bool;

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionTag {
    SubmitUrl,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Action {
    tag: ActionTag,
    associated: HashMap<String, String>,
}

impl Action {
    pub fn new(tag: ActionTag) -> Action {
        Action { tag, associated: HashMap::new() }
    }

    fn associate(mut self, name: impl Into<Cow<'static, str>>, prop: impl Into<Cow<'static, str>>) -> Action {
        self.associated.insert(name.into().into_owned(), prop.into().into_owned());
        self
    }
}

#[derive(Debug)]
enum Event {
    Ui(Action),
    Response {
        request: String,
        response: Result<CoapMessage, CoapError>,
    }
}

#[derive(Debug)]
struct State {
    handle: Handle,
    events: mpsc::Sender<Event>,
    session_log: Vec<SessionLog>,
}

impl State {
    fn spawn(handle: Handle) -> (impl Sink<SinkItem = Event, SinkError = ()>, impl Stream<Item = VNode<Action>, Error = ()>) {
        let (events_tx, events_rx) = mpsc::channel(1);
        let (render_tx, render_rx) = mpsc::channel(1);
        let mut state = State {
            handle: handle.clone(),
            events: events_tx.clone(),
            session_log: vec![],
        };
        handle.spawn(render_tx.send(state.render())
            .map_err(|e| println!("state send error: {:?}", e))
            .and_then(|tx| events_rx
                .fold(tx, move |tx, action| {
                    if state.update(action) {
                        Either::A(tx.send(state.render()).map_err(|e| println!("state send error: {:?}", e)))
                    } else {
                        Either::B(future::ok(tx))
                    }
                })
                .map(|_| ())));
        (events_tx.sink_map_err(|e| println!("error sinking event: {:?}", e)), render_rx)
    }

    fn update(&mut self, event: Event) -> ShouldRender {
        match event {
            Event::Ui(Action { tag: ActionTag::SubmitUrl, associated, .. }) => {
                let url = associated.get("value").unwrap().clone();
                let events = self.events.clone();
                self.session_log.insert(0, SessionLog::Request { url: url.clone() });
                self.handle.spawn(
                    Client::get(&url)
                        .send()
                        .then(|response| events.send(Event::Response {
                            request: url,
                            response,
                        }))
                        .map(|_| ())
                        .map_err(|e| println!("error sending response in: {:?}", e)))
            }
            Event::Response { request, response } => {
                self.session_log.insert(0, SessionLog::Response {
                    request,
                    response,
                });
            }
        }
        true
    }

    fn render(&self) -> VNode<Action> {
        VTag::new("div")
            .child(VTag::new("input")
                .prop("type", "text")
                .prop("placeholder", "coap url")
                .prop("onchange", VProperty::Action(
                        Action::new(ActionTag::SubmitUrl)
                            .associate("value", "value"))))
            .child(VTag::new("ol")
                .children(self.session_log.iter().map(|log|
                    VTag::new("li").child(log.render().map_action(&|a| a)))))
            .into()
    }
}

pub fn new(handle: Handle) -> (impl Sink<SinkItem = Action, SinkError = ()>, impl Stream<Item = VNode<Action>, Error = ()>) {
    let (events, renders) = State::spawn(handle.clone());
    let (ui, actions) = mpsc::channel(1);

    let events = events.with(|e| { println!("event: {:?}", e); e });
    let renders = renders.map(|r| { println!("render: {:?}", r); r });

    handle.spawn(actions.map(|action| Ok(Event::Ui(action))).forward(events).map(|_| ()));
    (ui.sink_map_err(|e| println!("error sinking action: {:?}", e)), renders)
}
