use std::fmt::Display;

use super::style::PrintError;

pub fn handle<T, E>(result: Result<T, E>, message: &str, cast_error: bool, quit: bool) -> Option<T>
where
    E: Display,
{
    match result {
        Ok(t) => Some(t),
        Err(e) => {
            PrintError(
                {
                    if cast_error {
                        format!("{}: {}", message, e)
                    } else {
                        message.to_string()
                    }
                },
                quit,
            )
            .ok();
            None
        }
    }
}
