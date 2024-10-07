use regex::Regex;
use serde_json::Value;
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
};

use ansi_term::Colour::{Blue, Green, Purple, Red, Yellow};

use super::extractor::MessageExtractor;

pub struct CoreServiceFormatter;

impl<S, N> FormatEvent<S, N> for CoreServiceFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> core::fmt::Result {
        // Get the metadata of the event
        let metadata = event.metadata();

        // Determine color and emoji based on level
        let (color, emoji) = match *metadata.level() {
            tracing::Level::ERROR => (Red, "❌"),
            tracing::Level::WARN => (Yellow, "⚠️"),
            tracing::Level::INFO => (Green, "ℹ️"),
            tracing::Level::DEBUG => (Blue, "🐛"),
            tracing::Level::TRACE => (Purple, "🔍"),
        };

        // Begin custom output formatting
        writeln!(writer, "┌──────────────────────────")?;

        // Error info (e.g., event level)
        write!(writer, "│ Level: ")?;
        writeln!(
            writer,
            "{} {}",
            color.paint(metadata.level().to_string()),
            emoji
        )?;
        writeln!(writer, "├──────────────────────────")?;

        // Method stack history or target
        write!(writer, "│ Target: ")?;
        writeln!(writer, "{}", metadata.target())?;
        writeln!(writer, "├──────────────────────────")?;

        // Log message
        write!(writer, "│ ")?;

        // Create a visitor to extract the message and process it
        let mut visitor = MessageExtractor::default();
        event.record(&mut visitor);

        // Process the message to remove sensitive information and add improved formatting
        let sanitized_message = sanitize_message(&visitor.message);

        // Write the sanitized message with improved formatting
        for line in sanitized_message.lines() {
            writeln!(writer, "│ {}", line)?;
        }

        writeln!(writer, "└──────────────────────────")?;

        Ok(())
    }
}

fn sanitize_message(message: &str) -> String {
    // First, sanitize the message to remove sensitive information
    let re_password = Regex::new(r#"(password\s*:\s*)(".*?"|\S+)"#).unwrap();
    let mut sanitized_message = re_password.replace_all(message, r#"${1}"***""#).to_string();

    // Now, extract and format headers
    if let Some(formatted_headers) = extract_and_format_headers(&sanitized_message) {
        // Replace the headers in the message with the formatted headers
        let re_headers = Regex::new(r"headers:\s*\{[^}]*\}").unwrap();
        sanitized_message = re_headers
            .replace(
                &sanitized_message,
                &format!("headers:{}", formatted_headers),
            )
            .to_string();
    }

    // Improve overall formatting by adding new lines to separate sections
    let re_metadata = Regex::new(r"(metadata:\s*MetadataMap)").unwrap();
    sanitized_message = re_metadata.replace(&sanitized_message, "\n$1").to_string();

    let re_message = Regex::new(r"(message:\s*AuthenticateWithPasswordRequest)").unwrap();
    sanitized_message = re_message.replace(&sanitized_message, "\n$1").to_string();

    let re_extensions = Regex::new(r"(extensions:\s*Extensions)").unwrap();
    sanitized_message = re_extensions
        .replace(&sanitized_message, "\n$1")
        .to_string();

    sanitized_message
}

fn extract_and_format_headers(message: &str) -> Option<String> {
    let re_headers = Regex::new(r"headers:\s*\{([^}]*)\}").unwrap();
    if let Some(caps) = re_headers.captures(message) {
        let headers_str = format!("{{{}}}", caps.get(1)?.as_str()); // Add braces to make it valid JSON

        // Parse headers as JSON
        match serde_json::from_str::<Value>(&headers_str) {
            Ok(json_value) => {
                // Pretty-print the JSON with indentation
                let formatted_headers = serde_json::to_string_pretty(&json_value).unwrap();

                // Indent the entire formatted headers for better readability
                let indented_headers = formatted_headers
                    .lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<String>>()
                    .join("\n");

                // Add surrounding new lines for better separation in the output
                Some(format!("\n{}\n", indented_headers))
            }
            Err(e) => {
                eprintln!("Error parsing headers as JSON: {}", e);
                None
            }
        }
    } else {
        None
    }
}
