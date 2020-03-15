use std::sync::Arc;

use tokio_coap::error::Error as CoapError;

use iced::{Element, Text, Column};

#[derive(Debug)]
pub struct Error {
    request: String,
    error: Arc<CoapError>,
}

impl Error {
    pub fn new(request: String, error: Arc<CoapError>) -> Self {
        Self {
            request,
            error,
        }
    }

    pub fn view(&mut self) -> Element<'_, !> {
        Column::new()
            .push(Text::new(format!("Error requesting {}", self.request)))
            .push(Text::new(format!("{:#?}", self.error)))
            .into()
    }
}
