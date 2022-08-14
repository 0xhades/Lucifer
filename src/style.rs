use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::{fmt::Display, process::exit, time::Duration};

use std::{
    error::Error,
    io::{stderr, stdin, stdout},
};

pub fn clear() -> Result<(), Box<dyn Error>> {
    execute!(stdout(), terminal::Clear(ClearType::All))?;
    execute!(stdout(), cursor::RestorePosition)?;
    Ok(())
}

pub fn PrintError<T>(
    error: T,
    quit: bool,
    primary: Color,
    secondary: Color,
) -> Result<(), Box<dyn Error>>
where
    T: Display,
{
    PrintColorful("[", secondary)?;
    PrintColorful("!", primary)?;
    PrintColorful("] ", secondary)?;

    PrintColorful(error.to_string().as_str(), primary)?;
    if quit {
        exit(1);
    }
    Ok(())
}

pub fn PrintlnError<T>(
    error: T,
    quit: bool,
    primary: Color,
    secondary: Color,
) -> Result<(), Box<dyn Error>>
where
    T: Display,
{
    PrintColorful("[", secondary)?;
    PrintColorful("!", primary)?;
    PrintColorful("] ", secondary)?;

    PrintColorful(
        format!("{}\n", error.to_string().as_str()).as_str(),
        primary,
    )?;
    if quit {
        exit(1);
    }
    Ok(())
}

pub fn PrintSuccess<T>(message: T, primary: Color, secondary: Color) -> Result<(), Box<dyn Error>>
where
    T: Display,
{
    PrintColorful("[", secondary)?;
    PrintColorful("+", primary)?;
    PrintColorful("] ", secondary)?;
    PrintColorful(message.to_string().as_str(), Color::Green)?;
    Ok(())
}

pub fn PrintlnSuccess<T>(message: T, primary: Color, secondary: Color) -> Result<(), Box<dyn Error>>
where
    T: Display,
{
    PrintColorful("[", secondary)?;
    PrintColorful("+", primary)?;
    PrintColorful("] ", secondary)?;
    PrintColorful(
        format!("{}\n", message.to_string().as_str()).as_str(),
        Color::Green,
    )?;
    Ok(())
}

pub fn PrintDescription(
    s: &str,
    description: &str,
    primary: Color,
    secondary: Color,
) -> Result<(), Box<dyn Error>> {
    PrintColorful("[", secondary)?;
    PrintColorful("+", primary)?;
    PrintColorful("] ", secondary)?;
    PrintColorful(&format!("{} (", s), primary)?;
    PrintColorful(description, secondary)?;
    PrintColorful(")\n", primary)?;
    Ok(())
}

pub fn PrintlnColorfulPlus(
    s: &str,
    primary: Color,
    secondary: Color,
) -> Result<(), Box<dyn Error>> {
    PrintColorful("[", secondary)?;
    PrintColorful("+", primary)?;
    PrintColorful("] ", secondary)?;
    PrintColorful(&format!("{}\n", s), primary)?;
    Ok(())
}

pub fn PrintColorful(s: &str, color: Color) -> Result<(), Box<dyn Error>> {
    execute!(stdout(), SetForegroundColor(color), Print(s), ResetColor)?;
    Ok(())
}

pub fn PrintlnColorful(s: &str, color: Color) -> Result<(), Box<dyn Error>> {
    execute!(
        stdout(),
        SetForegroundColor(color),
        Print(s),
        Print("\n"),
        ResetColor
    )?;
    Ok(())
}

pub fn user_input(
    enter: &str,
    primary: Color,
    secondary: Color,
    input_color: Color,
) -> Result<String, Box<dyn Error>> {
    PrintColorful("[", secondary)?;
    PrintColorful("+", primary)?;
    PrintColorful("] ", secondary)?;
    PrintColorful(&format!("{}: ", enter), primary)?;
    input(input_color)
}

pub fn user_input_description(
    enter: &str,
    description: &str,
    default: &str,
    primary: Color,
    secondary: Color,
    input_color: Color,
) -> Result<String, Box<dyn Error>> {
    PrintColorful("[", secondary)?;
    PrintColorful("+", primary)?;
    PrintColorful("] ", secondary)?;
    PrintColorful(&format!("{} (", enter), primary)?;
    PrintColorful(description, secondary)?;
    PrintColorful("): ", primary)?;

    match input(input_color) {
        Ok(t) if t.len() != 0 => Ok(t),
        Ok(_) | Err(_) => Ok(default.to_string()),
    }
}

pub async fn user_input_num(
    enter: &str,
    default: u32,
    primary: Color,
    secondary: Color,
    input_color: Color,
) -> Result<u32, Box<dyn Error>> {
    PrintColorful("[", secondary)?;
    PrintColorful("+", primary)?;
    PrintColorful("] ", secondary)?;
    PrintColorful(&format!("{}: ", enter), primary)?;

    input_num(default, input_color).await
}

pub async fn user_input_num_description(
    enter: &str,
    description: &str,
    default: u32,
    primary: Color,
    secondary: Color,
    input_color: Color,
) -> Result<u32, Box<dyn Error>> {
    PrintColorful("[", secondary)?;
    PrintColorful("+", primary)?;
    PrintColorful("] ", secondary)?;

    PrintColorful(&format!("{} (", enter), primary)?;
    PrintColorful(description, secondary)?;
    PrintColorful("): ", primary)?;

    input_num(default, input_color).await
}

pub async fn input_num(default: u32, input_color: Color) -> Result<u32, Box<dyn Error>> {
    loop {
        let s = input(input_color)?;
        if s == "" {
            return Ok(default);
        }
        match s.trim().parse::<u32>() {
            Ok(n) if n != 0 => return Ok(n),
            Ok(_) | Err(_) => {
                PrintColorful("[", Color::DarkRed)?;
                PrintColorful("!", Color::DarkCyan)?;
                PrintColorful("] ", Color::DarkRed)?;
                PrintColorful("Enter a valid number\r", Color::DarkCyan)?;
                tokio::time::sleep(Duration::from_secs(2)).await;
                execute!(stdout(), Print(" ".repeat(25)))?;
                continue;
            }
        }
    }
}

pub fn input(input_color: Color) -> Result<String, Box<dyn Error>> {
    let mut input = String::new();
    execute!(stdout(), SetForegroundColor(input_color))?;
    stdin().read_line(&mut input)?;
    execute!(stdout(), ResetColor)?;
    Ok(input)
}
