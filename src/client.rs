use std::collections::HashMap;

use vdom_rsjs::{VNode, VTag, VProperty};
use tokio_core::reactor::Handle;
use tokio_coap::Client;
use tokio_coap::message::Message as CoapMessage;
use tokio_coap::error::Error as CoapError;

use futures::{Sink, Stream, Future, future::{self, Either}};
use futures::unsync::mpsc;

type ShouldRender = bool;

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionTag {
    SubmitUrl,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Action {
    tag: ActionTag,
    data: (),
    associated: HashMap<String, String>,
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
    session_log: Vec<String>,
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
                self.session_log.push(format!("Request to {}", url));
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
                self.session_log.push(format!("Response to {}: {:?}", request, response));
            }
        }
        true
    }

    fn render(&self) -> VNode<Action> {
        VNode::Tag(VTag {
            name: "div".into(),
            properties: HashMap::new(),
            children: vec![
                VNode::Tag(VTag {
                    name: "input".into(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("type".into(), VProperty::Text("text".into()));
                        props.insert("placeholder".into(), VProperty::Text("coap url".into()));
                        props.insert("onchange".into(), VProperty::Action(Action {
                            tag: ActionTag::SubmitUrl,
                            data: (),
                            associated: {
                                let mut associated = HashMap::new();
                                associated.insert("value".into(), "value".into());
                                associated
                            },
                        }));
                        props
                    },
                    children: vec![],
                    key: None,
                    namespace: None,
                }),
                VNode::Tag(VTag {
                    name: "ol".into(),
                    properties: HashMap::new(),
                    children: self.session_log.iter().map(|log| {
                        VNode::Tag(VTag {
                            name: "li".into(),
                            properties: HashMap::new(),
                            children: vec![VNode::Text(log.clone())],
                            key: None,
                            namespace: None,
                        })
                    }).collect(),
                    key: None,
                    namespace: None,
                }),
            ],
            key: None,
            namespace: None,
        })
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
