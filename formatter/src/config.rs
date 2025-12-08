/// Configuration for the formatter
#[derive(Clone, Debug)]
pub struct FormatConfig {
    /// Maximum line width before breaking (default: 80)
    pub max_width: usize,
    /// Number of spaces per indentation level (default: 2)
    pub indent_size: usize,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            max_width: 80,
            indent_size: 4,
        }
    }
}

impl FormatConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_width = width;
        self
    }

    pub fn with_indent_size(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }
}
