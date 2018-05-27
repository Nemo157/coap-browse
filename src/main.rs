extern crate websocket;
extern crate futures;
extern crate tokio_core;
extern crate vdom_rsjs;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use tokio_core::reactor::Core;

mod component;
mod root;
mod serve;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let root = root::Root::new(handle.clone());
    let server = serve::serve(handle, root);

    core.run(server).unwrap();
}
