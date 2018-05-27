use std::fmt::Debug;

use websocket::message::OwnedMessage;
use websocket::server::InvalidConnection;
use websocket::async::Server;

use tokio_core::reactor::Handle;
use futures::{Future, Sink, Stream, stream};
use vdom_rsjs::VNode;
use serde_json;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct FullUpdate<A> {
    tree: VNode<A>,
}

pub fn serve<Action, ClientSink, ClientStream, NewClient>(handle: Handle, mut new_client: NewClient) -> impl Future<Item = (), Error = ()>
where Action: Serialize + for<'a> Deserialize<'a> + Debug,
      ClientSink: Sink<SinkItem = Action, SinkError = ()> + 'static,
      ClientStream: Stream<Item = VNode<Action>, Error = ()> + 'static,
      NewClient: FnMut() -> (ClientSink, ClientStream) + Clone + 'static,
{
    let server = Server::bind("127.0.0.1:8080", &handle).unwrap();

    server.incoming()
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, addr)| {
            println!("Got a connection from {}", addr);

            if !upgrade.protocols().iter().any(|s| s == "coap-browse") {
                spawn_future(upgrade.reject(), "Upgrade Rejection", &handle);
                return Ok(());
            }

            let (client_sink, client_stream) = new_client();
            let f = upgrade
                .use_protocol("coap-browse")
                .accept()
                .map_err(|e| println!("error accepting stream: {:?}", e))
                .and_then(move |(ws, _)| {
                    let (ws_sink, ws_stream) = ws.split();
                    let incoming = ws_stream
                        .take_while(|m| Ok(!m.is_close()))
                        .filter_map(|m| match m {
                            OwnedMessage::Ping(_) => {
                                // TODO: Handle pings, going to need to
                                // change these stream/sink pairs to have a
                                // multiplexer in between them to allow
                                // bypassing the client for sending the PONG
                                // response.
                                None
                            }
                            OwnedMessage::Pong(_) => None,
                            OwnedMessage::Text(msg) => {
                                match serde_json::from_str(&msg) {
                                    Ok(action) => Some(action),
                                    Err(err) => {
                                        println!("error deserializing {:?}", err);
                                        None
                                    }
                                }
                            }
                            OwnedMessage::Binary(_) => {
                                println!("unexpected binary message");
                                None
                            }
                            OwnedMessage::Close(_) => {
                                None
                            }
                        })
                        .map_err(|e| println!("error handling ws_stream: {:?}", e))
                        .forward(client_sink.sink_map_err(|e| println!("error handling client_sink: {:?}", e)));
                    let outgoing = client_stream
                        .map_err(|e| println!("error on client_stream: {:?}", e))
                        .map(|tree| serde_json::to_string(&FullUpdate { tree }).unwrap())
                        .map(|json| OwnedMessage::Text(json))
                        .chain(stream::once(Ok(OwnedMessage::Close(None))))
                        .forward(ws_sink.sink_map_err(|e| println!("error on ws_sink: {:?}", e)));
                    incoming.join(outgoing)
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
