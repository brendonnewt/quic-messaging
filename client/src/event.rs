/* Handles input events and updates app state */

use crossterm::event::{self, KeyCode, KeyEvent, Event};
use std::time::Duration;

pub fn poll_event() -> Result<Option<KeyEvent>, Box<dyn std::error::Error>> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key_event) = event::read()? {
            return Ok(Some(key_event));
        }
    }
    Ok(None)
}
