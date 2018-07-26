#![feature(never_type)]
#![feature(slice_patterns)]
#![feature(futures_api)]
#![feature(pin)]
#![feature(async_await)]
#![feature(await_macro)]
#![feature(arbitrary_self_types)]

use tokio_core::reactor::Core;
use futures::executor;
use std::boxed::PinBox;

mod client;
mod log;

fn main() {
    let core = Core::new().unwrap();
    let handle = core.handle();

    let server = vdom_websocket_rsjs::serve(handle.clone(), || PinBox::new(client::new()));

    executor::block_on(server);
}
