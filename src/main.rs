#![feature(never_type)]
#![feature(slice_patterns)]

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
