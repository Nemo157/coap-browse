#![feature(never_type)]
#![feature(slice_patterns)]

extern crate websocket;
extern crate futures;
extern crate tokio_core;
extern crate vdom_rsjs;
extern crate serde_json;
extern crate serde_cbor;
extern crate serde_cbor_diag;
extern crate serde_xml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tokio_coap;
extern crate im;

use tokio_core::reactor::Core;

mod client;
mod serve;
mod log;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let server = serve::serve(handle.clone(), move || client::new(handle.clone()));

    core.run(server).unwrap();
}
