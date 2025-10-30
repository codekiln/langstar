use crate::error::Result;
use colored::Colorize;
use serde::Serialize;
use tabled::{
    Table, Tabled,
    settings::{Modify, Width, object::Rows, style::Style},
};

/// Output format for displaying data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON output
    Json,
    /// Table output (human-readable)
    Table,
}

impl OutputFormat {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "table" => Ok(OutputFormat::Table),
            _ => Err(crate::error::CliError::Config(format!(
                "Invalid output format: {}. Valid formats: json, table",
                s
            ))),
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
        }
    }

    /// Print data as JSON
    fn print_json<T: Serialize>(&self, data: &T) -> Result<()> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| crate::error::CliError::Other(e.into()))?;
        println!("{}", json);
        Ok(())
    }

    /// Print a table
    pub fn print_table<T: Tabled>(&self, data: &[T]) -> Result<()> {
        if data.is_empty() {
            println!("No results found.");
            return Ok(());
        }

        let mut table = Table::new(data);
        table
            .with(Style::rounded())
            .with(Modify::new(Rows::first()).with(Width::wrap(80)));

        println!("{}", table);
        Ok(())
    }

    /// Print a success message
    #[allow(dead_code)]
    pub fn success(&self, message: &str) {
        println!("{} {}", "✓".green(), message);
    }

    /// Print an error message
    #[allow(dead_code)]
    pub fn error(&self, message: &str) {
        eprintln!("{} {}", "✗".red(), message);
    }

    /// Print a warning message
    #[allow(dead_code)]
    pub fn warning(&self, message: &str) {
        println!("{} {}", "⚠".yellow(), message);
    }

    /// Print an info message
    pub fn info(&self, message: &str) {
        println!("{} {}", "ℹ".blue(), message);
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
        assert_eq!(OutputFormat::from_str("JSON").unwrap(), OutputFormat::Json);
        assert!(OutputFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_formatter_json() {
        let formatter = OutputFormatter::new(OutputFormat::Json);
        let data = serde_json::json!({"test": "value"});
        assert!(formatter.print(&data).is_ok());
    }
}
