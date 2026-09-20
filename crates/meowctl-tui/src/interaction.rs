//! Asking the user something.
//!
//! Separate from rendering, because input and output in one type is what made
//! `Printer.Confirm` hard to test; see [R-TUI-060].

use std::io::{BufRead, Write};

/// A question that could not be answered.
#[derive(Debug, thiserror::Error)]
pub enum InteractionError {
    /// Nobody is there to answer.
    ///
    /// Failing rather than blocking. A CI run that hits a prompt otherwise
    /// waits until the job times out, and the log says nothing about why; see
    /// [R-TUI-062].
    #[error("{question} needs an answer, and this session is not interactive")]
    NotInteractive {
        /// What was going to be asked.
        question: String,
    },

    /// The terminal could not be read or written.
    #[error("asking {question}: {source}")]
    Io {
        /// What was being asked.
        question: String,
        /// Why it failed.
        source: std::io::Error,
    },
}

/// Asking the user a question.
pub trait Interaction {
    /// Asks a yes-or-no question, defaulting to no.
    ///
    /// # Errors
    ///
    /// [`InteractionError::NotInteractive`] when nobody can answer.
    fn confirm(&mut self, question: &str) -> Result<bool, InteractionError>;

    /// Asks a free-text question and returns what was typed.
    ///
    /// Separate from [`Interaction::confirm`] rather than underneath it: a
    /// confirmation has a default and a fixed vocabulary, and the session that
    /// cannot answer has to say which of the two it is refusing; see
    /// [R-TUI-063]. `ctx.prompt` is the caller.
    ///
    /// # Errors
    ///
    /// [`InteractionError::NotInteractive`] when nobody can answer.
    fn ask(&mut self, question: &str) -> Result<String, InteractionError>;
}

/// Asks on the real terminal.
///
/// The question goes to standard error, so a run whose standard output is
/// being piped is not corrupted by it; see [R-TUI-061].
#[derive(Debug)]
pub struct Prompt<R, W> {
    input: R,
    output: W,
    interactive: bool,
}

impl<R: BufRead, W: Write> Prompt<R, W> {
    /// A prompt reading from `input` and asking on `output`.
    ///
    /// `interactive` is whether anybody is there. A non-interactive prompt
    /// fails rather than reading, because reading would consume whatever the
    /// pipe held and treat it as an answer.
    pub const fn new(input: R, output: W, interactive: bool) -> Self {
        Prompt {
            input,
            output,
            interactive,
        }
    }
}

impl<R: BufRead, W: Write> Interaction for Prompt<R, W> {
    fn ask(&mut self, question: &str) -> Result<String, InteractionError> {
        if !self.interactive {
            return Err(InteractionError::NotInteractive {
                question: question.to_owned(),
            });
        }

        let io = |source| InteractionError::Io {
            question: question.to_owned(),
            source,
        };

        write!(self.output, "{question} ").map_err(io)?;
        self.output.flush().map_err(io)?;

        let mut answer = String::new();
        if self.input.read_line(&mut answer).map_err(io)? == 0 {
            // End of input is an empty answer, which is what `starPrompt`
            // returns, and the newline closes the line the question opened.
            writeln!(self.output).map_err(io)?;
            return Ok(String::new());
        }
        Ok(answer.trim_end_matches(['\r', '\n']).to_owned())
    }

    fn confirm(&mut self, question: &str) -> Result<bool, InteractionError> {
        if !self.interactive {
            return Err(InteractionError::NotInteractive {
                question: question.to_owned(),
            });
        }

        let io = |source| InteractionError::Io {
            question: question.to_owned(),
            source,
        };

        write!(self.output, "{question} [y/N] ").map_err(io)?;
        self.output.flush().map_err(io)?;

        let mut answer = String::new();
        let read = self.input.read_line(&mut answer).map_err(io)?;
        if read == 0 {
            // End of input is no, and the newline closes the line the
            // question opened.
            writeln!(self.output).map_err(io)?;
            return Ok(false);
        }

        // A bare Enter is no, which is what the advertised default says.
        // `fmt.Scanln` returned "unexpected newline" for it, so pressing Enter
        // failed the command instead of cancelling; that is what
        // `fix(cli): confirmation prompt failed on Enter` closed.
        Ok(matches!(
            answer.trim().to_ascii_lowercase().as_str(),
            "y" | "yes"
        ))
    }
}

/// Answers without asking, for a test and for `--yes`.
///
/// The free-text answer is empty, which is what end-of-input gives.
#[derive(Debug, Clone, Copy)]
pub struct Always(pub bool);

impl Interaction for Always {
    fn ask(&mut self, _question: &str) -> Result<String, InteractionError> {
        Ok(String::new())
    }

    fn confirm(&mut self, _question: &str) -> Result<bool, InteractionError> {
        Ok(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(input: &str) -> (Result<bool, InteractionError>, String) {
        let mut output = Vec::new();
        let answer = {
            let mut prompt = Prompt::new(input.as_bytes(), &mut output, true);
            prompt.confirm("Apply changes?")
        };
        (answer, String::from_utf8_lossy(&output).into_owned())
    }

    /// [R-TUI-060] and [R-TUI-061]: prompting goes through the trait rather
    /// than reading stdin, which is what lets this be tested from a string at
    /// all -- `Printer.Confirm` could not be. The advertised default is no,
    /// and pressing Enter has to mean it: `fmt.Scanln` returned "unexpected
    /// newline" for a bare Enter, so the command failed instead of
    /// cancelling.
    #[test]
    fn a_bare_enter_means_no() {
        let (answer, _) = ask("\n");
        assert!(!answer.expect("an answer"));
    }

    /// [R-TUI-061] and so does end of input.
    #[test]
    fn end_of_input_means_no() {
        let (answer, output) = ask("");
        assert!(!answer.expect("an answer"));
        assert!(
            output.ends_with('\n'),
            "the question was left open: {output:?}"
        );
    }

    /// Only an explicit yes means yes: anything else is a typo, and a typo
    /// that applied changes would be the wrong way round.
    #[test]
    fn only_an_explicit_yes_means_yes() {
        for input in ["y\n", "Y\n", "yes\n", " YES \n"] {
            assert!(ask(input).0.expect("an answer"), "{input:?}");
        }
        for input in ["n\n", "no\n", "maybe\n", "yep\n", "1\n"] {
            assert!(!ask(input).0.expect("an answer"), "{input:?}");
        }
    }

    /// [R-TUI-061] the question goes to standard error, so a run whose
    /// standard output is piped is not corrupted by it.
    #[test]
    fn the_question_says_what_the_default_is() {
        let (_, output) = ask("\n");
        assert!(output.contains("Apply changes?"), "{output:?}");
        assert!(output.contains("[y/N]"), "{output:?}");
    }

    /// [R-TUI-062] a CI run that hits a prompt otherwise waits until the job
    /// times out, and the log says nothing about why.
    #[test]
    fn a_non_interactive_session_fails_rather_than_blocking() {
        let mut output = Vec::new();
        let mut prompt = Prompt::new("y\n".as_bytes(), &mut output, false);

        let err = prompt.confirm("Apply changes?").expect_err("should refuse");
        assert!(
            matches!(err, InteractionError::NotInteractive { .. }),
            "{err:?}"
        );
        assert!(err.to_string().contains("Apply changes?"), "{err}");

        // Nothing was read: the pipe's contents are not an answer.
        assert!(output.is_empty(), "{output:?}");
    }

    /// [R-TUI-063] `ctx.prompt` returns what was typed, and the newline is
    /// not part of it.
    #[test]
    fn a_free_text_question_returns_the_line_without_its_newline() {
        let mut output = Vec::new();
        let answer = {
            let mut prompt = Prompt::new("Ada Lovelace\n".as_bytes(), &mut output, true);
            prompt.ask("Your name?")
        };
        assert_eq!(answer.expect("an answer"), "Ada Lovelace");
        let written = String::from_utf8_lossy(&output).into_owned();
        assert!(written.contains("Your name?"), "{written:?}");
        assert!(
            !written.contains("[y/N]"),
            "a free-text question is not a confirmation: {written:?}"
        );
    }

    /// [R-TUI-063] end of input is an empty answer, which is what `starPrompt`
    /// returns, rather than a failure.
    #[test]
    fn end_of_input_is_an_empty_answer() {
        let mut output = Vec::new();
        let answer = {
            let mut prompt = Prompt::new("".as_bytes(), &mut output, true);
            prompt.ask("Your name?")
        };
        assert_eq!(answer.expect("an answer"), "");
    }

    /// [R-TUI-062] and a free-text question refuses in a non-interactive
    /// session for the same reason a confirmation does.
    #[test]
    fn a_free_text_question_also_refuses_when_nobody_is_there() {
        let mut output = Vec::new();
        let mut prompt = Prompt::new("Ada\n".as_bytes(), &mut output, false);
        let err = prompt.ask("Your name?").expect_err("should refuse");
        assert!(
            matches!(err, InteractionError::NotInteractive { .. }),
            "{err:?}"
        );
        assert!(output.is_empty(), "nothing was read: {output:?}");
    }

    /// `--yes` answers without asking.
    #[test]
    fn a_fixed_answer_does_not_ask() {
        assert!(Always(true).confirm("anything").expect("an answer"));
        assert!(!Always(false).confirm("anything").expect("an answer"));
    }
}
