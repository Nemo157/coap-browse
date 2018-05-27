use std::fmt::Debug;
use std::net::SocketAddr;

use websocket::message::{Message, OwnedMessage};
use websocket::server::InvalidConnection;
use websocket::async::Server;

use tokio_core::reactor::Handle;
use futures::{Future, Sink, Stream};
use vdom_rsjs::VNode;
use serde_json;
use serde::{Serialize, Deserialize};

use component::Component;

#[derive(Serialize, Deserialize, Debug)]
struct FullUpdate<A> {
    tree: VNode<A>,
}

fn handle_message<A, C>(addr: &SocketAddr, root: &mut C, msg: OwnedMessage) -> Option<OwnedMessage>
where A: Serialize + for<'a> Deserialize<'a> + Debug, C: Component<Action = A> + Debug {
    match msg {
        OwnedMessage::Ping(msg) => Some(OwnedMessage::Pong(msg)),
        OwnedMessage::Pong(_) => None,
        OwnedMessage::Text(msg) => {
            let action: A = match serde_json::from_str(&msg) {
                Ok(action) => action,
                Err(err) => {
                    println!("{}: error deserializing {:?}", addr, err);
                    return None;
                }
            };
            println!("{}: new action: {:?}", addr, action);
            println!("{}: new state: {:?}", addr, root);
            if root.update(action) {
                let tree = root.render();
                println!("{}: new tree: {:?}", addr, tree);
                let json = serde_json::to_string(&FullUpdate { tree }).unwrap();
                Some(OwnedMessage::Text(json))
            } else {
                println!("{}: no change", addr);
                None
            }
        }
        OwnedMessage::Binary(_) => {
            println!("{}: unexpected binary message", addr);
            None
        }
        OwnedMessage::Close(_) => {
            None
        }
    }
}

pub fn serve<A, C>(handle: Handle, root: C) -> impl Future<Item = (), Error = ()>
where A: Serialize + for<'a> Deserialize<'a> + Debug, C: Component<Action = A> + Debug + Clone + 'static {
    let server = Server::bind("127.0.0.1:8080", &handle).unwrap();

    server.incoming()
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, addr)| {
            let root = root.clone();

            println!("Got a connection from {}", addr);

            if !upgrade.protocols().iter().any(|s| s == "coap-browse") {
                spawn_future(upgrade.reject(), "Upgrade Rejection", &handle);
                return Ok(());
            }

            let f = upgrade
                .use_protocol("coap-browse")
                .accept()
                .and_then(move |(s, _)| {
                    let tree = root.render();
                    println!("{}: initial state: {:?}", addr, root);
                    println!("{}: initial tree: {:?}", addr, tree);

                    let json = serde_json::to_string(&FullUpdate { tree }).unwrap();
                    s.send(Message::text(json).into())
                        .and_then(|s| Ok((s, root)))
                })
                .and_then(move |(s, mut root)| {
                    let (sink, stream) = s.split();
                    stream
                        .take_while(|m| Ok(!m.is_close()))
                        .filter_map(move |m| handle_message(&addr, &mut root, m))
                        .forward(sink)
                        .and_then(|(_, sink)| {
                            sink.send(OwnedMessage::Close(None))
                        })
                });

            spawn_future(f, "Client Status", &handle);
            Ok(())
        })
        .map_err(|err| println!("Server error: {:?}", err))
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
    where F: Future<Item = I, Error = E> + 'static,
          E: Debug
{
    handle.spawn(f.map_err(move |e| println!("{}: '{:?}'", desc, e))
                  .map(move |_| println!("{}: Finished.", desc)));
}
