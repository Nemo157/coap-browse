use std::collections::HashMap;

use vdom_rsjs::{VNode, VTag, VProperty};
use tokio_core::reactor::Handle;

use futures::{Sink, Stream, Future, future::{self, Either}};
use futures::unsync::mpsc;

type ShouldRender = bool;

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionTag {
    Increment,
    Decrement,
    SubmitMsg,
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
}

#[derive(Debug)]
struct State {
    handle: Handle,
    count: usize,
    msg: String,
}

impl State {
    fn spawn(handle: Handle) -> (impl Sink<SinkItem = Event, SinkError = ()>, impl Stream<Item = VNode<Action>, Error = ()>) {
        let (events_tx, events_rx) = mpsc::channel(1);
        let (render_tx, render_rx) = mpsc::channel(1);
        let mut state = State { handle: handle.clone(), count: 0, msg: "".into() };
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
            Event::Ui(Action { tag: ActionTag::Increment, .. })
                => self.count += 1,
            Event::Ui(Action { tag: ActionTag::Decrement, .. })
                => self.count -= 1,
            Event::Ui(Action { tag: ActionTag::SubmitMsg, associated, .. })
                => self.msg = associated.get("value").unwrap().clone(),
        }
        true
    }

    fn render(&self) -> VNode<Action> {
        VNode::Tag(VTag {
            name: "div".into(),
            properties: HashMap::new(),
            children: vec![
                VNode::Text(self.count.to_string()),
                VNode::Tag(VTag {
                    name: "br".into(),
                    properties: HashMap::new(),
                    children: vec![],
                    key: None,
                    namespace: None,
                }),
                VNode::Tag(VTag {
                    name: "button".into(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("onclick".into(), VProperty::Action(Action {
                            tag: ActionTag::Increment,
                            data: (),
                            associated: HashMap::new(),
                        }));
                        props
                    },
                    children: vec![
                        VNode::Text("increment".into()),
                    ],
                    key: None,
                    namespace: None,
                }),
                VNode::Tag(VTag {
                    name: "button".into(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("onclick".into(), VProperty::Action(Action {
                            tag: ActionTag::Decrement,
                            data: (),
                            associated: HashMap::new(),
                        }));
                        props
                    },
                    children: vec![
                        VNode::Text("decrement".into()),
                    ],
                    key: None,
                    namespace: None,
                }),
                VNode::Tag(VTag {
                    name: "input".into(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("type".into(), VProperty::Text("text".into()));
                        props.insert("onchange".into(), VProperty::Action(Action {
                            tag: ActionTag::SubmitMsg,
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
                VNode::Text(self.msg.clone()),
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
