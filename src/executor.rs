use std::{io, future::Future};
use tokio_compat::runtime::Runtime;
use iced::Executor;

pub struct TokioCompat {
    inner: Runtime,
}

impl Executor for TokioCompat {
    fn new() -> io::Result<Self> {
        try { Self { inner: Runtime::new()? } }
    }

    fn spawn(&self, fut: impl Future<Output = ()> + Send + 'static) {
        self.inner.spawn_std(fut);
    }
}

