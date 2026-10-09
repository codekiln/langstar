//! Shared confirmation prompt for destructive commands.

use crate::error::{CliError, Result};
use std::io::{self, BufRead, IsTerminal, Write};

/// Ask the user to confirm a destructive action.
///
/// Returns `Ok(true)` when the answer is one of `accepted` (case-insensitive),
/// `Ok(false)` for any other answer or end of input.
///
/// When stdin is not a terminal there is nobody to ask, so this returns an
/// error naming `skip_flag` instead of hanging or deleting silently.
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

/// Does the work of [`confirm`], taking the input source, the is-a-terminal answer and the
/// place to print the prompt as arguments so the unit tests can supply them.
pub fn confirm_from(
    input: &mut impl BufRead,
    is_tty: bool,
    prompt_out: &mut impl Write,
    question: &str,
    accepted: &[&str],
    skip_flag: &str,
) -> Result<bool> {
    if !is_tty {
        return Err(CliError::Config(format!(
            "confirmation required but stdin is not a terminal; pass {} to proceed without prompting",
            skip_flag
        )));
    }

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
    fn prompt_is_written() {
        let mut out = Vec::new();
        confirm_from(&mut "n\n".as_bytes(), true, &mut out, "Q?", YN, "--yes").unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "Q? ");
    }
}
