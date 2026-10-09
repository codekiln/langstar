//! Shared confirmation prompt for destructive commands.

use crate::error::{CliError, Result};
use std::io::{self, BufRead, IsTerminal, Write};

/// Ask the user to confirm a destructive action.
///
/// Returns `Ok(true)` when the answer is one of `accepted` (case-insensitive),
/// `Ok(false)` for any other answer or end of input.
///
/// When stdin is not a terminal, nobody can answer, so this returns an error that names `skip_flag`.
pub fn confirm(question: &str, accepted: &[&str], skip_flag: &str) -> Result<bool> {
    let stdin = io::stdin();
    confirm_from(
        &mut stdin.lock(),
        stdin.is_terminal(),
        &mut io::stderr(),
        question,
        accepted,
        skip_flag,
    )
}

/// Fails when stdin is not a terminal, naming `skip_flag` as the way to proceed.
///
/// A command that must look something up over the network before it can ask calls this
/// first, so a script without the flag gets this error and not a lookup failure.
pub fn require_terminal(skip_flag: &str) -> Result<()> {
    require_terminal_if(io::stdin().is_terminal(), skip_flag)
}

fn require_terminal_if(is_tty: bool, skip_flag: &str) -> Result<()> {
    if is_tty {
        Ok(())
    } else {
        Err(CliError::Config(format!(
            "confirmation required but stdin is not a terminal; pass {} to proceed without prompting",
            skip_flag
        )))
    }
}

/// Asks the same question as [`confirm`], but the caller says where the answer is read from,
/// whether that source is a terminal, and where the question is printed. The unit tests use it
/// to run the prompt without a real terminal.
pub fn confirm_from(
    input: &mut impl BufRead,
    is_tty: bool,
    prompt_out: &mut impl Write,
    question: &str,
    accepted: &[&str],
    skip_flag: &str,
) -> Result<bool> {
    require_terminal_if(is_tty, skip_flag)?;

    write!(prompt_out, "{} ", question)?;
    prompt_out.flush()?;

    let mut line = String::new();
    input.read_line(&mut line)?;
    let answer = line.trim().to_lowercase();
    Ok(accepted.contains(&answer.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const YN: &[&str] = &["y", "yes"];

    fn run(input: &str, is_tty: bool, accepted: &[&str]) -> Result<bool> {
        confirm_from(
            &mut input.as_bytes(),
            is_tty,
            &mut Vec::new(),
            "Delete? [y/N]:",
            accepted,
            "--yes",
        )
    }

    #[test]
    fn accepts_y_and_yes_case_insensitive() {
        assert!(run("y\n", true, YN).unwrap());
        assert!(run(" YES \n", true, YN).unwrap());
    }

    #[test]
    fn declines_n_empty_and_eof() {
        assert!(!run("n\n", true, YN).unwrap());
        assert!(!run("\n", true, YN).unwrap());
        assert!(!run("", true, YN).unwrap());
        assert!(!run("maybe\n", true, YN).unwrap());
    }

    #[test]
    fn strict_mode_rejects_bare_y() {
        assert!(!run("y\n", true, &["yes"]).unwrap());
        assert!(run("yes\n", true, &["yes"]).unwrap());
    }

    #[test]
    fn non_tty_errors_naming_flag_even_if_input_says_yes() {
        let err = run("y\n", false, YN).unwrap_err().to_string();
        assert!(err.contains("--yes"), "{err}");
        assert!(err.contains("not a terminal"), "{err}");
    }

    #[test]
    fn require_terminal_errors_naming_flag_only_without_tty() {
        let err = require_terminal_if(false, "--force")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("--force") && err.contains("not a terminal"),
            "{err}"
        );
        assert!(require_terminal_if(true, "--force").is_ok());
    }

    #[test]
    fn prompt_is_written() {
        let mut out = Vec::new();
        confirm_from(&mut "n\n".as_bytes(), true, &mut out, "Q?", YN, "--yes").unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "Q? ");
    }
}
