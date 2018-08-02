use std::sync::{Arc, Mutex};

use vdom_rsjs::{VNode, VTag, VProperty};
use vdom_rsjs::render::{Render, Cache, TopCache};
use vdom_websocket_rsjs::Action;
use tokio_coap::Client;
use tokio_coap::message::Message as CoapMessage;
use tokio_coap::error::Error as CoapError;
use im::ConsList;
use serde_derive::{Deserialize, Serialize};

use futures::{Sink, Stream, Future, FutureExt, SinkExt, StreamExt, future::self};
use futures::compat::Future01CompatExt;
use futures::channel::mpsc;
use futures::executor;

use crate::log::SessionLog;

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
    events: mpsc::Sender<Event>,
    session_log: ConsList<SessionLog>,
}

impl State {
    fn spawn() -> impl Future<Output = (impl Sink<SinkItem = Event, SinkError = ()>, impl Stream<Item = Arc<VNode<Action<ActionTag>>>>)> {
        let (events_tx, mut events_rx) = mpsc::channel(1);
        let (mut render_tx, render_rx) = mpsc::channel(1);
        let cache = Mutex::new(TopCache::new());
        let mut state = Arc::new(State {
            events: events_tx.clone(),
            session_log: ConsList::new(),
        });
        async {
            await!(executor::spawn(async move {
                await!(render_tx.send(cache.lock().unwrap().render(state.clone()))).expect("Unexpected state send error");
                while let Some(event) = await!(events_rx.next()) {
                    let new_state = await!(state.update(event));
                    if !Arc::ptr_eq(&state, &new_state) {
                        state = new_state;
                        await!(render_tx.send(cache.lock().unwrap().render(state.clone()))).expect("Unexpected state send error");
                    }
                }
            }));
            (events_tx.sink_map_err(|e| println!("error sinking event: {:?}", e)), render_rx)
        }
    }

    fn update(&self, event: Event) -> impl Future<Output = Arc<State>> {
        let (events, session_log) = (self.events.clone(), self.session_log.clone());
        async move {
            match event {
                Event::Ui(Action { tag: ActionTag::SubmitUrl, associated, .. }) => {
                    let url = associated.get("value").unwrap().clone();
                    {
                        let mut events = events.clone();
                        let url = url.clone();
                        await!(executor::spawn(async move {
                            let client = Client::get(&url).expect("Good url");
                            let response = await!(client.send().compat());
                            let event = Event::Response {
                                request: url,
                                response,
                            };
                            await!(events.send(event)).expect("unexpected error sending response in")
                        }))
                    }
                    Arc::new(State {
                        session_log: session_log.cons(SessionLog::Request { url }),
                        events,
                    })
                }
                Event::Response { request, response } => {
                    Arc::new(State {
                        session_log: session_log.cons(SessionLog::Response {
                            request,
                            response,
                        }),
                        events,
                    })
                }
            }
        }
    }
}

impl Render<Action<ActionTag>> for State {
    fn render(&self, cache: &mut Cache<Action<ActionTag>>) -> VNode<Action<ActionTag>> {
        VTag::new("div")
            .prop("style", "display:flex;flex-direction:column;width:auto;height:auto;margin:10px 20px;border:1px solid #586e75;border-radius:3px")
            .child(VTag::new("div")
                .prop("style", "display:flex;flex-direction:row;background-color:#eee8d5;border-bottom:1px solid #586e75")
                .child(VTag::new("input")
                    .prop("type", "text")
                    .prop("placeholder", "coap url")
                    .prop("onchange", VProperty::Action(
                            Action::new(ActionTag::SubmitUrl)
                                .associate("value", "value")))))
            .child(VTag::new("div")
                .prop("style", "display:flex;flex-direction:column;list-style:none")
                .children(self.session_log.iter().map(|log| cache.render(log))))
            .into()
    }
}

pub fn new() -> impl Future<Output = (impl Sink<SinkItem = Action<ActionTag>, SinkError = ()>, impl Stream<Item = Arc<VNode<Action<ActionTag>>>>)> {
    State::spawn()
        .then(|(events, renders)| {
            let (ui, actions) = mpsc::channel(1);

            let events = events.with(|e| { println!("event: {:?}", e); future::ready(Ok(e)) });
            let renders = renders.map(|r| { println!("render: {:?}", r); r });

            executor::spawn(actions.map(|action| Ok::<Event, ()>(Event::Ui(action))).forward(events).map(|_| ()))
                .map(|()| {
                    (ui.sink_map_err(|e| println!("error sinking action: {:?}", e)), renders)
                })
        })
}
