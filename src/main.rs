#![feature(never_type)]
#![feature(slice_patterns)]
#![feature(futures_api)]
#![feature(pin)]
#![feature(async_await)]
#![feature(await_macro)]
#![feature(arbitrary_self_types)]

#![type_length_limit="2097152"]

use tokio_core::reactor::Core;
use std::boxed::PinBox;
use futures::future::{FutureExt, TryFutureExt};
use futures::compat::TokioDefaultExecutor;

mod client;
mod log;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let server = vdom_websocket_rsjs::serve(handle.clone(), || PinBox::new(client::new()));

    // executor::block_on(server);
    core.run(PinBox::new(server.map(Ok::<_, ()>)).compat(TokioDefaultExecutor)).unwrap();
}
