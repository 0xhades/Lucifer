use std::fmt::Display;

use crossterm::style::Color;

use super::style::PrintlnError;

pub fn handle<T, E>(result: Result<T, E>, message: &str, cast_error: bool, quit: bool) -> Option<T>
where
    E: Display,
{
    match result {
        Ok(t) => Some(t),
        Err(e) => {
            PrintlnError(
                {
                    if cast_error {
                        format!("{}: {}", message, e)
                    } else {
                        message.to_string()
                    }
                },
                quit,
                Color::Red,
                Color::Cyan,
            )
            .ok();
            None
        }
    }
}

