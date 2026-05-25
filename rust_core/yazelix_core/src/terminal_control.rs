use crossterm::{
    cursor::{MoveTo, MoveToColumn, MoveUp},
    queue,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

fn command_string(write_commands: impl FnOnce(&mut Vec<u8>) -> io::Result<()>) -> String {
    crossterm::style::force_color_output(true);
    let mut output = Vec::new();
    write_commands(&mut output).expect("crossterm command writes to memory");
    String::from_utf8(output).expect("crossterm commands emit UTF-8")
}

pub(crate) fn styled(text: impl ToString, color: Color) -> String {
    command_string(|output| {
        queue!(
            output,
            SetForegroundColor(color),
            Print(text.to_string()),
            SetForegroundColor(Color::Reset)
        )?;
        Ok(())
    })
}

pub(crate) fn styled_bold(text: impl ToString, color: Color) -> String {
    command_string(|output| {
        queue!(
            output,
            SetAttribute(Attribute::Bold),
            SetForegroundColor(color),
            Print(text.to_string()),
            SetAttribute(Attribute::NormalIntensity),
            SetForegroundColor(Color::Reset)
        )?;
        Ok(())
    })
}

pub(crate) fn styled_dim(text: impl ToString, color: Color) -> String {
    command_string(|output| {
        queue!(
            output,
            SetAttribute(Attribute::Dim),
            SetForegroundColor(color),
            Print(text.to_string()),
            SetAttribute(Attribute::NormalIntensity),
            SetForegroundColor(Color::Reset)
        )?;
        Ok(())
    })
}

pub(crate) fn styled_dim_default(text: impl ToString) -> String {
    command_string(|output| {
        queue!(
            output,
            SetAttribute(Attribute::Dim),
            Print(text.to_string()),
            SetAttribute(Attribute::NormalIntensity)
        )?;
        Ok(())
    })
}

pub(crate) fn styled_dim_no_reset(text: impl ToString, color: Color) -> String {
    command_string(|output| {
        queue!(
            output,
            SetAttribute(Attribute::Dim),
            SetForegroundColor(color),
            Print(text.to_string()),
            SetAttribute(Attribute::NormalIntensity),
            SetForegroundColor(Color::Reset)
        )?;
        Ok(())
    })
}

#[cfg(test)]
pub(crate) fn normal_foreground_sequence() -> String {
    command_string(|output| {
        queue!(
            output,
            SetAttribute(Attribute::NormalIntensity),
            SetForegroundColor(Color::Reset)
        )?;
        Ok(())
    })
}

#[cfg(test)]
pub(crate) fn reset_style_sequence() -> String {
    command_string(|output| {
        queue!(output, SetAttribute(Attribute::Reset))?;
        Ok(())
    })
}

#[cfg(test)]
pub(crate) fn true_black_background_sequence() -> String {
    command_string(|output| {
        queue!(output, SetBackgroundColor(Color::Rgb { r: 0, g: 0, b: 0 }))?;
        Ok(())
    })
}

#[cfg(test)]
pub(crate) fn default_background_clear_sequence() -> String {
    command_string(|output| {
        queue!(
            output,
            SetBackgroundColor(Color::Reset),
            MoveTo(0, 0),
            Clear(ClearType::All)
        )?;
        Ok(())
    })
}

pub(crate) fn clear_current_line_println_sequence(line: &str) -> String {
    command_string(|output| {
        queue!(
            output,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            Print(line),
            Print("\n")
        )?;
        Ok(())
    })
}

pub(crate) fn clear_screen_newline_sequence() -> String {
    command_string(|output| {
        queue!(output, MoveTo(0, 0), Clear(ClearType::All), Print("\n"))?;
        Ok(())
    })
}

pub(crate) fn move_up_sequence(rows: usize) -> String {
    command_string(|output| {
        queue!(output, MoveUp(rows.min(u16::MAX as usize) as u16))?;
        Ok(())
    })
}

pub(crate) fn clear_screen_now() -> io::Result<()> {
    crossterm::style::force_color_output(true);
    let mut stdout = io::stdout();
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    stdout.flush()
}

pub(crate) fn set_true_black_background_now() -> io::Result<()> {
    crossterm::style::force_color_output(true);
    let mut stdout = io::stdout();
    queue!(stdout, SetBackgroundColor(Color::Rgb { r: 0, g: 0, b: 0 }))?;
    stdout.flush()
}

pub(crate) fn clear_screen_with_default_background_now() -> io::Result<()> {
    crossterm::style::force_color_output(true);
    let mut stdout = io::stdout();
    queue!(
        stdout,
        SetBackgroundColor(Color::Reset),
        MoveTo(0, 0),
        Clear(ClearType::All)
    )?;
    stdout.flush()
}
