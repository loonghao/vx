//! Interactive input utilities.
//!
//! This module provides functions for interactive user input,
//! inspired by uv's console module.

use crate::{ConsoleError, Result};
use console::Term;
use std::io::Write;

/// Ask for confirmation from the user.
///
/// Returns `Ok(true)` if the user confirms, `Ok(false)` if they decline,
/// or `Err(ConsoleError::Cancelled)` if they press Ctrl+C.
///
/// # Example
/// ```rust,no_run
/// use vx_console::confirm;
///
/// let confirmed = confirm("Proceed with installation?", true).unwrap();
/// if confirmed {
///     println!("Installing...");
/// }
/// ```
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    let mut term = Term::stderr();

    let default_hint = if default { "[Y/n]" } else { "[y/N]" };
    write!(term, "? {} {} ", prompt, default_hint).map_err(ConsoleError::Io)?;
    term.flush().map_err(ConsoleError::Io)?;

    loop {
        let key = term.read_key().map_err(ConsoleError::Io)?;

        match key {
            console::Key::Char('y') | console::Key::Char('Y') => {
                writeln!(term, "y").map_err(ConsoleError::Io)?;
                return Ok(true);
            }
            console::Key::Char('n') | console::Key::Char('N') => {
                writeln!(term, "n").map_err(ConsoleError::Io)?;
                return Ok(false);
            }
            console::Key::Enter => {
                writeln!(term, "{}", if default { "y" } else { "n" }).map_err(ConsoleError::Io)?;
                return Ok(default);
            }
            console::Key::Escape => {
                writeln!(term).map_err(ConsoleError::Io)?;
                return Err(ConsoleError::Cancelled);
            }
            _ => {
                // Ignore other keys
            }
        }
    }
}

/// Read a password from the user (hidden input).
///
/// # Example
/// ```rust,no_run
/// use vx_console::password;
///
/// let token = password("Enter API token:").unwrap();
/// ```
pub fn password(prompt: &str) -> Result<String> {
    let mut term = Term::stderr();

    write!(term, "{} ", prompt).map_err(ConsoleError::Io)?;
    term.flush().map_err(ConsoleError::Io)?;

    let password = term.read_secure_line().map_err(ConsoleError::Io)?;
    Ok(password)
}

/// Read a line of input from the user.
///
/// # Example
/// ```rust,no_run
/// use vx_console::input;
///
/// let name = input("Project name:").unwrap();
/// ```
pub fn input(prompt: &str) -> Result<String> {
    let mut term = Term::stderr();

    write!(term, "{} ", prompt).map_err(ConsoleError::Io)?;
    term.flush().map_err(ConsoleError::Io)?;

    let line = term.read_line().map_err(ConsoleError::Io)?;
    Ok(line)
}

/// Present a selection menu to the user.
///
/// Returns the index of the selected item.
///
/// # Example
/// ```rust,no_run
/// use vx_console::select;
///
/// let options = &["npm", "yarn", "pnpm"];
/// let choice = select("Choose package manager:", options).unwrap();
/// println!("Selected: {}", options[choice]);
/// ```
pub fn select(prompt: &str, options: &[&str]) -> Result<usize> {
    let mut term = Term::stderr();

    writeln!(term, "? {}", prompt).map_err(ConsoleError::Io)?;

    let mut selected = 0;

    loop {
        // Clear and redraw options
        for (i, option) in options.iter().enumerate() {
            if i == selected {
                writeln!(term, "  \x1b[36m❯\x1b[0m {}", option).map_err(ConsoleError::Io)?;
            } else {
                writeln!(term, "    {}", option).map_err(ConsoleError::Io)?;
            }
        }

        let key = term.read_key().map_err(ConsoleError::Io)?;

        match key {
            console::Key::ArrowUp | console::Key::Char('k') => {
                selected = selected.saturating_sub(1);
            }
            console::Key::ArrowDown | console::Key::Char('j') => {
                if selected < options.len() - 1 {
                    selected += 1;
                }
            }
            console::Key::Enter => {
                return Ok(selected);
            }
            console::Key::Escape => {
                return Err(ConsoleError::Cancelled);
            }
            _ => {}
        }

        // Move cursor up to redraw
        for _ in 0..options.len() {
            term.move_cursor_up(1).map_err(ConsoleError::Io)?;
            term.clear_line().map_err(ConsoleError::Io)?;
        }
    }
}

/// Present a multi-select menu to the user.
///
/// Returns the indices of the selected items.
///
/// # Example
/// ```rust,no_run
/// use vx_console::multi_select;
///
/// let options = &["node", "python", "go", "rust"];
/// let choices = multi_select("Select tools to install:", options).unwrap();
/// for i in choices {
///     println!("Installing: {}", options[i]);
/// }
/// ```
pub fn multi_select(prompt: &str, options: &[&str]) -> Result<Vec<usize>> {
    let mut term = Term::stderr();

    writeln!(term, "? {} (space to toggle, enter to confirm)", prompt).map_err(ConsoleError::Io)?;

    let mut selected = 0;
    let mut checked: Vec<bool> = vec![false; options.len()];

    loop {
        // Clear and redraw options
        for (i, option) in options.iter().enumerate() {
            let check = if checked[i] { "◉" } else { "○" };
            if i == selected {
                writeln!(term, "  \x1b[36m❯\x1b[0m {} {}", check, option)
                    .map_err(ConsoleError::Io)?;
            } else {
                writeln!(term, "    {} {}", check, option).map_err(ConsoleError::Io)?;
            }
        }

        let key = term.read_key().map_err(ConsoleError::Io)?;

        match key {
            console::Key::ArrowUp | console::Key::Char('k') => {
                selected = selected.saturating_sub(1);
            }
            console::Key::ArrowDown | console::Key::Char('j') => {
                if selected < options.len() - 1 {
                    selected += 1;
                }
            }
            console::Key::Char(' ') => {
                checked[selected] = !checked[selected];
            }
            console::Key::Enter => {
                let result: Vec<usize> = checked
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &c)| if c { Some(i) } else { None })
                    .collect();
                return Ok(result);
            }
            console::Key::Escape => {
                return Err(ConsoleError::Cancelled);
            }
            _ => {}
        }

        // Move cursor up to redraw
        for _ in 0..options.len() {
            term.move_cursor_up(1).map_err(ConsoleError::Io)?;
            term.clear_line().map_err(ConsoleError::Io)?;
        }
    }
}

#[cfg(test)]
mod tests {
    // Interactive tests are difficult to automate
    // These would typically be tested manually or with a mock terminal
}
