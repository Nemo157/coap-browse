use std::collections::HashMap;

use vdom_rsjs::{VNode, VTag, VProperty};
use tokio_core::reactor::Handle;

use futures::{Sink, Stream, Future, future::{self, Either}};
use futures::unsync::mpsc;

type ShouldRender = bool;

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Increment,
    Decrement,
}

#[derive(Debug)]
struct State {
    count: usize,
}

impl State {
    fn with_channels(mut self, rx: mpsc::Receiver<Action>, tx: mpsc::Sender<VNode<Action>>) -> impl Future<Item = (), Error = ()> {
        tx.send(self.render())
            .map_err(|e| println!("state send error: {:?}", e))
            .and_then(|tx| rx
                .fold(tx, move |tx, action| {
                    if self.update(action) {
                        Either::A(tx.send(self.render()).map_err(|e| println!("state send error: {:?}", e)))
                    } else {
                        Either::B(future::ok(tx))
                    }
                })
                .map(|_| ()))
    }

    fn update(&mut self, action: Action) -> ShouldRender {
        match action {
            Action::Increment => self.count += 1,
            Action::Decrement => self.count -= 1,
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
                        props.insert("onclick".into(), VProperty::Action(Action::Increment));
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
                        props.insert("onclick".into(), VProperty::Action(Action::Decrement));
                        props
                    },
                    children: vec![
                        VNode::Text("decrement".into()),
                    ],
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
    let state = State { count: 0 };
    let (incoming_tx, incoming_rx) = mpsc::channel(1);
    let (outgoing_tx, outgoing_rx) = mpsc::channel(1);

    handle.spawn(state.with_channels(incoming_rx, outgoing_tx));
    (incoming_tx.sink_map_err(|e| println!("error sinking action: {:?}", e)), outgoing_rx)
}
