//! Utilities for prompting the user in the terminal.

use crate::utils::trim_newline;
use anyhow::Context;
use crossterm::style::Stylize;
use std::io;
use std::io::Write;

const INDENT: &str = "    ";
const ACTION_HELP: &str = "[s]kip : Don't create the symlink and move on to the next one.
[S]kip all : [s]kip for the current symlink and all further symlink conflicting with an existing file.
[b]ackup : Move the existing file in BACKUP_DIR, then make the current symlink.
[B]ackup all : [b]ackup for the current symlink and all further symlink conflicting with an existing file.
[o]verwrite : Overwrite the existing file with the symlink (beware data loss!)
[O]verwrite all : [o]verwrite for the current symlink and all further symlink conflicting with an existing file.";

fn get_stdin_line_input() -> anyhow::Result<String> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .with_context(|| "Error reading stdin input.")?;
    // Need this because the newline of Enter is included in the input
    trim_newline(&mut input);

    Ok(input)
}

trait PromptOptions {
    fn match_input(input: &str) -> Option<Self>
    where
        Self: Sized;
    fn get_valid_inputs() -> Vec<String>;
}

fn prompt_option<PO: PromptOptions>(
    mess: &str,
    help_input: Option<&str>,
    help_mess: Option<&str>,
) -> anyhow::Result<PO> {
    let has_help = help_input.is_some() && help_mess.is_some();
    let help_input = help_input.unwrap_or("");
    let help_mess = help_mess.unwrap_or("");

    loop {
        print!("{}", mess);
        io::stdout().flush()?;
        let input = get_stdin_line_input()?;

        if let Some(opt) = PO::match_input(&input) {
            return Ok(opt);
        } else if has_help && input == help_input {
            println!("{INDENT}----------");
            for line in help_mess.lines() {
                println!("{INDENT}{}", line);
            }
            println!("{INDENT}----------");
        } else {
            let mut help_key = String::from("");
            if has_help {
                help_key = format!(", {}", help_input);
            }
            println!(
                "{INDENT}Wrong input! Valid inputs are: {}{}. Try again.",
                PO::get_valid_inputs().join(", "),
                help_key,
            );
        }
    }
}

struct ErrorPromptOptions {}

impl PromptOptions for ErrorPromptOptions {
    fn match_input(input: &str) -> Option<Self> {
        let _ = input;
        Some(ErrorPromptOptions {})
    }

    fn get_valid_inputs() -> Vec<String> {
        vec![]
    }
}

/// Prompts the user to continue after an error occured, forcing him
/// to acknowledge it.
///
/// Outputs are to stdout and input received from stdin.
///
/// # Parameters
///
/// - `err_mess`: The error message to show the user.
///
/// # Errors
///
/// Fails if reading/writing from/to stdin/stdout fails.
///
/// # Examples
///
/// ```rust,no_run
/// use mksls::prompt;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// prompt::error_prompt("The error message...")?;
/// # Ok(())
/// # }
/// ```
pub fn error_prompt(err_mess: &str) -> anyhow::Result<()> {
    let prompt_mess = format!(
        "(?) {}\n{}Enter a key to continue: ",
        err_mess.red(),
        INDENT
    );
    let _ = prompt_option::<ErrorPromptOptions>(&prompt_mess, None, None)?;

    Ok(())
}

/// Options the user can choose when confronted to a conflict that prevents
/// the creation of a symlink.
pub enum AlreadyExistPromptOptions {
    /// Don't create the symlink and move on to the next one.
    Skip,
    /// Skip for the current symlink and all further symlink conflicting with an existing file.
    AlwaysSkip,
    /// Move the existing file in BACKUP_DIR, then make the current symlink.
    Backup,
    /// Backup for the current symlink and all further symlink conflicting with an existing file.
    AlwaysBackup,
    /// Overwrite the existing file with the symlink (beware data loss!).
    Overwrite,
    /// Overwrite for the current symlink and all further symlink conflicting with an existing file.
    AlwaysOverwrite,
}

impl PromptOptions for AlreadyExistPromptOptions {
    fn match_input(input: &str) -> Option<Self> {
        match input {
            "s" => Some(AlreadyExistPromptOptions::Skip),
            "S" => Some(AlreadyExistPromptOptions::AlwaysSkip),
            "b" => Some(AlreadyExistPromptOptions::Backup),
            "B" => Some(AlreadyExistPromptOptions::AlwaysBackup),
            "o" => Some(AlreadyExistPromptOptions::Overwrite),
            "O" => Some(AlreadyExistPromptOptions::AlwaysOverwrite),
            _ => None,
        }
    }

    fn get_valid_inputs() -> Vec<String> {
        vec![
            String::from("s"),
            String::from("S"),
            String::from("b"),
            String::from("B"),
            String::from("o"),
            String::from("O"),
        ]
    }
}

/// Prompts the user to choose one of the [`AlreadyExistPromptOptions`] when
/// faced with a conflict preventing the creation of the desired symlink.
///
/// # Parameters
///
/// - `target_path_str`: A string representation of the target's path.
/// - `link_path_str`: A string representation of the link's path.
///
/// # Returns
///
/// The option chosen by the user, or an error if reading/writing from/to
/// stdin/stdout failed.
///
/// # Examples
///
/// ```rust,no_run
/// use mksls::prompt;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// prompt::already_exist_prompt("/.../target", "/.../link")?;
/// # Ok(())
/// # }
/// ```
pub fn already_exist_prompt(
    target_path_str: &str,
    link_path_str: &str,
) -> anyhow::Result<AlreadyExistPromptOptions> {
    let prompt_mess = format!(
        "(?) {} -> {}
{}A file already exists at link path.
{}[s]kip [S]kip all [b]ackup [B]ackup all [o]verwrite [O]verwrite all [h]elp: ",
        link_path_str.red(),
        target_path_str,
        INDENT,
        INDENT
    );
    let input =
        prompt_option::<AlreadyExistPromptOptions>(&prompt_mess, Some("h"), Some(ACTION_HELP))?;

    Ok(input)
}
