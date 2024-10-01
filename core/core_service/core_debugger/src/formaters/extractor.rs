// Visitor to extract the message from the event
use std::fmt::Debug;
use tracing::field::{Field, Visit};

#[derive(Default)]
pub struct MessageExtractor {
    pub message: String,
}

impl Visit for MessageExtractor {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            // Remove surrounding quotes from the Debug representation
            let value_str = format!("{:?}", value);
            self.message = value_str.trim_matches('"').to_string();
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}
