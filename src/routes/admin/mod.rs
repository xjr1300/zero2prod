mod dashboard;
mod logout;
mod newsletter;
mod password;

pub use dashboard::*;
pub use logout::*;
pub use newsletter::{publish_newsletter, publish_newsletter_form};
pub use password::*;

use std::fmt::Write;

use actix_web_flash_messages::IncomingFlashMessages;

pub fn flash_messages_html(flash_messages: IncomingFlashMessages) -> String {
    let mut messages = String::new();
    for m in flash_messages.iter() {
        writeln!(messages, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    messages
}
