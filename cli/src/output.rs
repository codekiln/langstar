use crate::error::Result;
use clap::ValueEnum;
use colored::Colorize;
use serde::Serialize;
use tabled::{
    Table, Tabled,
    settings::{Modify, Width, object::Columns, style::Style},
};
use terminal_size::{Width as TermWidth, terminal_size};

/// Default terminal width when detection fails
const DEFAULT_TERMINAL_WIDTH: usize = 80;

/// Get the current terminal width, falling back to default if detection fails
pub fn get_terminal_width() -> usize {
    terminal_size()
        .map(|(TermWidth(w), _)| w as usize)
        .unwrap_or(DEFAULT_TERMINAL_WIDTH)
}

/// Trait for types that can provide column metadata for text output
///
/// Commands that implement this trait can support column selection via `--columns`
/// and column discovery via `--show-columns`.
#[allow(dead_code)]
pub trait ColumnMetadata {
    /// Returns the list of available column names for this type
    ///
    /// Column names should be lowercase and use underscores (e.g., "repo_handle", "num_downloads")
    fn available_columns() -> Vec<&'static str>;

    /// Renders this item as tab-separated values for the given columns
    ///
    /// # Arguments
    /// * `columns` - The list of columns to include in the output
    ///
    /// # Returns
    /// A string with tab-separated values for the requested columns
    ///
    /// # Example
    /// ```ignore
    /// let columns = vec!["handle".to_string(), "downloads".to_string()];
    /// let tsv = item.render_tsv(&columns);
    /// assert_eq!(tsv, "my-prompt\t123");
    /// ```
    fn render_tsv(&self, columns: &[String]) -> String;
}

/// Output format for displaying data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON output
    Json,
    /// Table output (human-readable)
    Table,
    /// Text output (tab-separated values)
    Text,
}

impl OutputFormat {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "table" => Ok(OutputFormat::Table),
            "text" => Ok(OutputFormat::Text),
            _ => Err(crate::error::CliError::Config(format!(
                "Invalid output format: {}. Valid formats: json, table, text",
                s
            ))),
        }
    }
}

/// Export format for evaluation results and dataset examples
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum ExportFormat {
    /// CSV format
    Csv,
    /// JSON Lines format
    Jsonl,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Csv => write!(f, "csv"),
            ExportFormat::Jsonl => write!(f, "jsonl"),
        }
    }
}

/// Output formatter for CLI results
pub struct OutputFormatter {
    format: OutputFormat,
}

impl OutputFormatter {
    /// Create a new formatter with the given format
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Print data to stdout
    pub fn print<T: Serialize>(&self, data: &T) -> Result<()> {
        match self.format {
            OutputFormat::Json => self.print_json(data),
            OutputFormat::Table => {
                // For table format, the type needs to implement Tabled
                // For now, we'll fall back to JSON for types that don't implement Tabled
                self.print_json(data)
            }
            OutputFormat::Text => {
                // For text format, the type needs to implement ColumnMetadata
                // For now, we'll fall back to JSON for types that don't implement ColumnMetadata
                self.print_json(data)
            }
        }
    }

    /// Print data as JSON
    fn print_json<T: Serialize>(&self, data: &T) -> Result<()> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| crate::error::CliError::Other(e.into()))?;
        println!("{}", json);
        Ok(())
    }

    /// Print a table with automatic width adjustment based on terminal size
    pub fn print_table<T: Tabled>(&self, data: &[T]) -> Result<()> {
        self.print_table_with_options(data, None)
    }

    /// Print a table with configurable first column max width
    ///
    /// # Arguments
    /// * `data` - The data to display
    /// * `first_col_max_width` - Optional max width for first column (e.g., Handle).
    ///   If None, uses dynamic calculation based on terminal width.
    pub fn print_table_with_options<T: Tabled>(
        &self,
        data: &[T],
        first_col_max_width: Option<usize>,
    ) -> Result<()> {
        if data.is_empty() {
            println!("No results found.");
            return Ok(());
        }

        let term_width = get_terminal_width();

        // Calculate first column max width:
        // Reserve ~45 chars for other columns + borders, rest for first column
        // Minimum of 30, maximum of 100 (for very wide terminals)
        let first_col_width = first_col_max_width.unwrap_or_else(|| {
            let available = term_width.saturating_sub(45);
            available.clamp(30, 100)
        });

        let mut table = Table::new(data);
        table.with(Style::rounded()).with(
            Modify::new(Columns::first()).with(Width::truncate(first_col_width).suffix("...")),
        );

        println!("{}", table);
        Ok(())
    }

    /// Helper method to print a message with conditional routing based on format
    ///
    /// In JSON mode, outputs to stderr to keep stdout clean for machine-readable JSON.
    /// This follows the Unix/CLI convention used by cargo, git, curl, etc.
    fn print_message(&self, icon: impl std::fmt::Display, message: &str) {
        let formatted = format!("{} {}", icon, message);
        if self.format == OutputFormat::Json {
            eprintln!("{}", formatted);
        } else {
            println!("{}", formatted);
        }
    }

    /// Print a success message
    ///
    /// In JSON mode, outputs to stderr to keep stdout clean for machine-readable JSON.
    /// This follows the Unix/CLI convention used by cargo, git, curl, etc.
    #[allow(dead_code)]
    pub fn success(&self, message: &str) {
        self.print_message("✓".green(), message);
    }

    /// Print an error message
    ///
    /// Error messages always go to stderr regardless of output format.
    #[allow(dead_code)]
    pub fn error(&self, message: &str) {
        eprintln!("{} {}", "✗".red(), message);
    }

    /// Print a warning message
    ///
    /// In JSON mode, outputs to stderr to keep stdout clean for machine-readable JSON.
    /// This follows the Unix/CLI convention used by cargo, git, curl, etc.
    #[allow(dead_code)]
    pub fn warning(&self, message: &str) {
        self.print_message("⚠".yellow(), message);
    }

    /// Print an info message
    ///
    /// In JSON mode, outputs to stderr to keep stdout clean for machine-readable JSON.
    /// This follows the Unix/CLI convention used by cargo, git, curl, etc.
    pub fn info(&self, message: &str) {
        self.print_message("ℹ".blue(), message);
    }

    /// Print text output (tab-separated values)
    ///
    /// # Arguments
    /// * `data` - The data to display (must implement ColumnMetadata)
    /// * `columns` - The columns to include in the output. If None, uses all available columns.
    ///
    /// # Example
    /// ```ignore
    /// let formatter = OutputFormatter::new(OutputFormat::Text);
    /// formatter.print_text(&items, Some(&["handle".to_string(), "downloads".to_string()]));
    /// ```
    #[allow(dead_code)]
    pub fn print_text<T: ColumnMetadata>(
        &self,
        data: &[T],
        columns: Option<&[String]>,
    ) -> Result<()> {
        if data.is_empty() {
            println!("No results found.");
            return Ok(());
        }

        // Use provided columns or default to all available columns
        let cols = if let Some(c) = columns {
            c.to_vec()
        } else {
            T::available_columns()
                .iter()
                .map(|s| s.to_string())
                .collect()
        };

        // Render each item as TSV
        for item in data {
            println!("{}", item.render_tsv(&cols));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(
            OutputFormat::from_str("table").unwrap(),
            OutputFormat::Table
        );
        assert_eq!(OutputFormat::from_str("text").unwrap(), OutputFormat::Text);
        assert_eq!(OutputFormat::from_str("JSON").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("TEXT").unwrap(), OutputFormat::Text);
        assert!(OutputFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_formatter_json() {
        let formatter = OutputFormatter::new(OutputFormat::Json);
        let data = serde_json::json!({"test": "value"});
        assert!(formatter.print(&data).is_ok());
    }

    #[test]
    fn test_get_terminal_width() {
        let width = get_terminal_width();
        // In CI environments without a TTY, terminal_size() returns None,
        // so we fall back to DEFAULT_TERMINAL_WIDTH (80).
        // Real terminals may be wider, but never narrower than this minimum.
        assert!(width >= DEFAULT_TERMINAL_WIDTH);
    }
}
