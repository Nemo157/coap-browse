extern crate websocket;
extern crate futures;
extern crate tokio_core;
extern crate vdom_rsjs;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use tokio_core::reactor::Core;

mod client;
mod serve;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let server = serve::serve(handle.clone(), move || client::new(handle.clone()));

    core.run(server).unwrap();
}
