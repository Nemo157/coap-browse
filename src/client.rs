use std::sync::Arc;

use vdom_rsjs::{VNode, VTag, VProperty};
use vdom_rsjs::render::{Render, Cache, TopCache};
use vdom_websocket_rsjs::Action;
use tokio_core::reactor::Handle;
use tokio_coap::Client;
use tokio_coap::message::Message as CoapMessage;
use tokio_coap::error::Error as CoapError;
use im::ConsList;

use futures::{Sink, Stream, Future, future::{self, Either}};
use futures::unsync::mpsc;

use log::SessionLog;

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionTag {
    SubmitUrl,
}

#[derive(Debug)]
enum Event {
    Ui(Action<ActionTag>),
    Response {
        request: String,
        response: Result<CoapMessage, CoapError>,
    }
}

#[derive(Debug)]
struct State {
    handle: Handle,
    events: mpsc::Sender<Event>,
    session_log: ConsList<SessionLog>,
}

impl State {
    fn spawn(handle: Handle) -> (impl Sink<SinkItem = Event, SinkError = ()>, impl Stream<Item = Arc<VNode<Action<ActionTag>>>, Error = ()>) {
        let (events_tx, events_rx) = mpsc::channel(1);
        let (render_tx, render_rx) = mpsc::channel(1);
        let mut cache = TopCache::new();
        let mut state = Arc::new(State {
            handle: handle.clone(),
            events: events_tx.clone(),
            session_log: ConsList::new(),
        });
        handle.spawn(render_tx.send(cache.render(state.clone()))
            .map_err(|e| println!("state send error: {:?}", e))
            .and_then(|tx| events_rx
                .fold(tx, move |tx, action| {
                    let new_state = state.update(action);
                    if !Arc::ptr_eq(&state, &new_state) {
                        state = new_state;
                        Either::A(tx.send(cache.render(state.clone())).map_err(|e| println!("state send error: {:?}", e)))
                    } else {
                        Either::B(future::ok(tx))
                    }
                })
                .map(|_| ())));
        (events_tx.sink_map_err(|e| println!("error sinking event: {:?}", e)), render_rx)
    }

    fn update(&self, event: Event) -> Arc<State> {
        match event {
            Event::Ui(Action { tag: ActionTag::SubmitUrl, associated, .. }) => {
                let url = associated.get("value").unwrap().clone();
                let events = self.events.clone();
                {
                    let url = url.clone();
                    self.handle.spawn(
                        future::result(Client::get(&url))
                            .and_then(|client| client.send())
                            .then(|response| events.send(Event::Response {
                                request: url,
                                response,
                            }))
                            .map(|_| ())
                            .map_err(|e| println!("error sending response in: {:?}", e)));
                }
                Arc::new(State {
                    session_log: self.session_log.cons(SessionLog::Request { url }),
                    handle: self.handle.clone(),
                    events: self.events.clone(),
                })
            }
            Event::Response { request, response } => {
                Arc::new(State {
                    session_log: self.session_log.cons(SessionLog::Response {
                        request,
                        response,
                    }),
                    handle: self.handle.clone(),
                    events: self.events.clone(),
                })
            }
        }
    }
}

impl Render<Action<ActionTag>> for State {
    fn render(&self, cache: &mut Cache<Action<ActionTag>>) -> VNode<Action<ActionTag>> {
        VTag::new("div")
            .child(VTag::new("style").child(include_str!("style.css")))
            .child(VTag::new("main")
                .child(VTag::new("div")
                    .prop("className", "toolbar")
                    .child(VTag::new("input")
                        .prop("type", "text")
                        .prop("placeholder", "coap url")
                        .prop("onchange", VProperty::Action(
                                Action::new(ActionTag::SubmitUrl)
                                    .associate("value", "value")))))
                .child(VTag::new("div")
                    .prop("className", "log")
                    .children(self.session_log.iter().map(|log| cache.render(log)))))
            .into()
    }
}

pub fn new(handle: Handle) -> (impl Sink<SinkItem = Action<ActionTag>, SinkError = ()>, impl Stream<Item = Arc<VNode<Action<ActionTag>>>, Error = ()>) {
    let (events, renders) = State::spawn(handle.clone());
    let (ui, actions) = mpsc::channel(1);

    let events = events.with(|e| { println!("event: {:?}", e); e });
    let renders = renders.map(|r| { println!("render: {:?}", r); r });

    handle.spawn(actions.map(|action| Ok(Event::Ui(action))).forward(events).map(|_| ()));
    (ui.sink_map_err(|e| println!("error sinking action: {:?}", e)), renders)
}
