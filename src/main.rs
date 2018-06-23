#![feature(never_type)]
#![feature(slice_patterns)]

#[macro_use]
extern crate serde_derive;

extern crate futures;
extern crate im;
extern crate serde;
extern crate serde_cbor;
extern crate serde_cbor_diag;
extern crate serde_json;
extern crate serde_xml;
extern crate tokio_coap;
extern crate tokio_core;
extern crate vdom_rsjs;
extern crate vdom_websocket_rsjs;
extern crate websocket;

use tokio_core::reactor::Core;

mod client;
mod log;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let server = vdom_websocket_rsjs::serve(
        handle.clone(),
        move || client::new(handle.clone()));

    core.run(server).unwrap();
}
