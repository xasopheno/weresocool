use colored::*;
use std::io::Write;

/// Unified error display for all parser errors
pub struct ErrorDisplay<'a> {
    pub source: &'a str,
    pub line: usize,
    pub column: usize,
    pub label: &'a str,
    pub use_cyan: bool,  // true = cyan/red, false = yellow/red
}

impl<'a> ErrorDisplay<'a> {
    /// Display the error with colored output
    pub fn display(&self, quiet: bool) {
        if quiet {
            return;
        }

        // Signal error to TUI FIRST to clear old content
        println!("!");
        std::io::stdout().flush().ok();

        let start_offset: usize = 125;
        let end_offset: usize = 50;

        // Find the byte offset for the error position
        let error_pos = self.find_error_position();

        // Calculate display window
        let feed_start = error_pos.saturating_sub(start_offset);
        let mut feed_end = (error_pos + end_offset).min(self.source.len());
        if feed_end - feed_start > 300 {
            feed_end = feed_start + 300;
        }

        // Show context with colors
        let before = &self.source[feed_start..error_pos];
        let after = &self.source[error_pos..feed_end];

        if self.use_cyan {
            println!("{}{}", before.cyan(), after.red());
            println!(
                "
            {}
            {} at line {}
            {}
            ",
                "working".cyan().underline(),
                self.label,
                self.line.to_string().red().bold(),
                "broken".red().underline(),
            );
        } else {
            println!("{}{}", before.yellow(), after.red());
            println!(
                "
            {}
            {} at line {}
            {}
            ",
                "working".yellow().underline(),
                self.label,
                self.line.to_string().red().bold(),
                "broken".red().underline(),
            );
        }
        // Flush to ensure error output is sent before any signal
        std::io::stdout().flush().ok();
    }

    fn find_error_position(&self) -> usize {
        // Find the byte offset in source for the error line and column
        let mut current_line = 0;
        let mut line_start = 0;

        for (i, c) in self.source.char_indices() {
            if c == '\n' {
                current_line += 1;
                if current_line == self.line {
                    line_start = i + 1;
                    break;
                }
            }
        }

        line_start + self.column.saturating_sub(1)
    }
}
