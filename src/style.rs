use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::{fmt::Display, process::exit};

use std::{
    error::Error,
    io::{stderr, stdout},
};

pub fn clear() -> Result<(), Box<dyn Error>> {
    execute!(stdout(), terminal::Clear(ClearType::All))?;
    execute!(stdout(), cursor::RestorePosition)?;
    Ok(())
}

pub fn PrintError<T>(error: T, quit: bool) -> Result<(), Box<dyn Error>>
where
    T: Display,
{
    PrintColorful(error.to_string().as_str(), Color::Red)?;
    if quit {
        exit(1);
    }
    Ok(())
}

pub fn PrintSuccess<T>(message: T) -> Result<(), Box<dyn Error>>
where
    T: Display,
{
    PrintColorfulPlus(message.to_string().as_str(), Color::Cyan, Color::Yellow)
}

pub fn PrintColorless(s: &str) -> Result<(), Box<dyn Error>> {
    execute!(stdout(), Print(s), ResetColor)?;
    Ok(())
}

pub fn PrintColorfulPlus(s: &str, primary: Color, secondary: Color) -> Result<(), Box<dyn Error>> {
    execute!(
        stdout(),
        SetForegroundColor(primary),
        Print("["),
        ResetColor,
        SetForegroundColor(secondary),
        Print("+"),
        ResetColor,
        SetForegroundColor(primary),
        Print("] "),
        ResetColor,
        SetForegroundColor(primary),
        Print(s),
        ResetColor
    )?;

    Ok(())
}

pub fn PrintColorful(s: &str, color: Color) -> Result<(), Box<dyn Error>> {
    execute!(stdout(), SetForegroundColor(color), Print(s), ResetColor)?;
    Ok(())
}
