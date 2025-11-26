//! Format AST types - a parallel tree structure for source preservation
//!
//! The FormatNode tree captures spans and structure from the source code,
//! allowing the formatter to copy source text directly rather than
//! reconstructing it from the semantic AST.
//!
//! Key design principles:
//! - FormatNode is independent of Term - it mirrors the syntactic structure
//! - Each node has a span (byte offsets into source)
//! - The formatter uses span + source to copy original text (like "9.0", "Fm", "{1, 0, 1, 0}")
//! - Multiline detection: `span.contains_newline(source)`

use weresocool_ast::Term;

/// Span in source code (byte offsets)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Extract the source text for this span
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        if self.end <= source.len() {
            &source[self.start..self.end]
        } else {
            ""
        }
    }

    /// Check if the spanned source contains a newline
    pub fn contains_newline(&self, source: &str) -> bool {
        self.text(source).contains('\n')
    }

    /// Check if span is valid (non-empty and start <= end)
    pub fn is_valid(&self) -> bool {
        self.start < self.end
    }

    /// Merge two spans (take min start, max end)
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// FormatNode - captures syntactic structure with spans for source preservation
///
/// This is a parallel tree to the semantic AST (Term/Op), but captures
/// the exact source syntax including:
/// - Which keyword variant was used (Fm vs Tm)
/// - Whether braces or parens were used
/// - Whether content was multiline
/// - Literal number formats (9.0 vs 9 vs 9/1)
#[derive(Clone, Debug)]
pub enum FormatNode {
    /// A rational/number literal - span captures exact source text ("9.0", "1/2", "3")
    Rational { span: Span },

    /// A keyword with its value (e.g., "Fm 2" - keyword_span is "Fm", value is the FormatNode for "2")
    Keyword {
        keyword_span: Span,  // Just the keyword like "Fm"
        value: Box<FormatNode>,
        full_span: Span,     // The whole thing "Fm 2"
    },

    /// A name/identifier
    Name { span: Span },

    /// Compose/pipe chain: captures the delimiter style and items
    Compose {
        span: Span,
        children: Vec<FormatNode>,
        /// True if this was written with braces {}, false for bare pipe chain
        braced: bool,
    },

    /// Braced overtone shorthand: {f, a, g, p}
    BracedOvertone {
        span: Span,
        f: Box<FormatNode>,
        a: Box<FormatNode>,
        g: Box<FormatNode>,
        p: Box<FormatNode>,
    },

    /// Parenthesized overtone: (f, a, g, p)
    ParenOvertone {
        span: Span,
        f: Box<FormatNode>,
        a: Box<FormatNode>,
        g: Box<FormatNode>,
        p: Box<FormatNode>,
    },

    /// Sequence: Seq[...] or Sequence[...]
    Sequence {
        span: Span,
        keyword_span: Span,  // "Seq" or "Sequence"
        children: Vec<FormatNode>,
    },

    /// Overlay: Overlay[...] or O[...]
    Overlay {
        span: Span,
        keyword_span: Span,  // "Overlay" or "O"
        children: Vec<FormatNode>,
    },

    /// Choose
    Choose {
        span: Span,
        children: Vec<FormatNode>,
    },

    /// ModulateBy
    ModulateBy {
        span: Span,
        operations: Vec<FormatNode>,
        output: Option<Vec<FormatNode>>,
    },

    /// Function definition
    FunDef {
        span: Span,
        name_span: Span,
        args: Vec<FormatNode>,
        body: Box<FormatNode>,
    },

    /// Function call
    FunctionCall {
        span: Span,
        name_span: Span,
        args: Vec<FormatNode>,
    },

    /// Lambda
    Lambda {
        span: Span,
        input_name: Option<Span>,
        body: Box<FormatNode>,
    },

    /// WGSL block - span captures the entire wgsl { ... } including braces
    Wgsl { span: Span },

    /// Simple op with no children (AsIs, Noise, etc.)
    Simple { span: Span },

    /// Definition: name = { ... }
    Definition {
        span: Span,
        name_span: Span,
        value: Box<FormatNode>,
    },

    /// Init block: { f: ..., l: ..., g: ..., p: ... }
    Init {
        span: Span,
        f: Box<FormatNode>,
        l: Box<FormatNode>,
        g: Box<FormatNode>,
        p: Box<FormatNode>,
    },

    /// Caret shorthand: 2^1 meaning Fm 2 | Lm 1
    CaretShorthand {
        span: Span,
        freq: Box<FormatNode>,
        length: Box<FormatNode>,
    },

    /// Repeat: captures the count
    Repeat {
        span: Span,
        count_span: Span,
    },

    /// Filter ops (Lowpass, Highpass, Bandpass)
    Filter {
        span: Span,
        keyword_span: Span,
        cutoff: Box<FormatNode>,
        q: Box<FormatNode>,
    },

    /// List operations
    List {
        span: Span,
        children: Vec<FormatNode>,
    },

    /// Generator operations
    Generator {
        span: Span,
        // TODO: capture generator-specific structure
    },

    /// Color
    Color {
        span: Span,
    },

    /// Fallback for nodes we haven't specialized yet
    /// The span allows copying source text directly
    Fallback { span: Span },
}

impl FormatNode {
    /// Get the span for any FormatNode variant
    pub fn span(&self) -> Span {
        match self {
            FormatNode::Rational { span } => *span,
            FormatNode::Keyword { full_span, .. } => *full_span,
            FormatNode::Name { span } => *span,
            FormatNode::Compose { span, .. } => *span,
            FormatNode::BracedOvertone { span, .. } => *span,
            FormatNode::ParenOvertone { span, .. } => *span,
            FormatNode::Sequence { span, .. } => *span,
            FormatNode::Overlay { span, .. } => *span,
            FormatNode::Choose { span, .. } => *span,
            FormatNode::ModulateBy { span, .. } => *span,
            FormatNode::FunDef { span, .. } => *span,
            FormatNode::FunctionCall { span, .. } => *span,
            FormatNode::Lambda { span, .. } => *span,
            FormatNode::Wgsl { span } => *span,
            FormatNode::Simple { span } => *span,
            FormatNode::Definition { span, .. } => *span,
            FormatNode::Init { span, .. } => *span,
            FormatNode::CaretShorthand { span, .. } => *span,
            FormatNode::Repeat { span, .. } => *span,
            FormatNode::Filter { span, .. } => *span,
            FormatNode::List { span, .. } => *span,
            FormatNode::Generator { span, .. } => *span,
            FormatNode::Color { span } => *span,
            FormatNode::Fallback { span } => *span,
        }
    }

    /// Get the source text for this node
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        self.span().text(source)
    }

    /// Check if this node spans multiple lines in source
    pub fn is_multiline(&self, source: &str) -> bool {
        self.span().contains_newline(source)
    }
}

// ============================================================================
// Legacy types for backwards compatibility (will be removed in Phase 4)
// ============================================================================

/// A Term with formatting metadata (LEGACY - use FormatNode instead)
#[derive(Clone, Debug)]
pub struct FormatTerm {
    /// The underlying semantic term
    pub term: Term,
    /// Span in source (byte offsets)
    pub span: Span,
    /// For collections: were items written on separate lines in the source?
    pub multiline: bool,
    /// Children (for nested structures)
    pub children: Vec<FormatTerm>,
}

impl FormatTerm {
    /// Create a new FormatTerm with default (non-multiline) formatting
    pub fn new(term: Term, span: Span) -> Self {
        Self {
            term,
            span,
            multiline: false,
            children: Vec::new(),
        }
    }

    /// Create with multiline flag
    pub fn with_multiline(term: Term, span: Span, multiline: bool) -> Self {
        Self {
            term,
            span,
            multiline,
            children: Vec::new(),
        }
    }

    /// Create with children
    pub fn with_children(term: Term, span: Span, multiline: bool, children: Vec<FormatTerm>) -> Self {
        Self {
            term,
            span,
            multiline,
            children,
        }
    }
}

/// A definition with formatting metadata (LEGACY)
#[derive(Clone, Debug)]
pub struct FormatDef {
    pub name: String,
    pub value: FormatTerm,
    pub span: Span,
}

/// Result from parsing for the formatter
///
/// Contains both the semantic AST (for the app) and format tree (for formatting)
#[derive(Clone, Debug)]
pub struct FormatParseResult {
    /// The clean semantic parsed composition
    pub composition: weresocool_parser::ParsedComposition,
    /// The format tree (parallel to AST but with spans)
    pub format_tree: Option<FormatNode>,
    /// Definitions with format info (LEGACY)
    pub defs: Vec<FormatDef>,
    /// The original source (needed for span-based text extraction)
    pub source: String,
}

impl FormatParseResult {
    pub fn new(composition: weresocool_parser::ParsedComposition, source: String) -> Self {
        Self {
            composition,
            format_tree: None,
            defs: Vec::new(),
            source,
        }
    }

    pub fn with_format_tree(
        composition: weresocool_parser::ParsedComposition,
        format_tree: FormatNode,
        source: String,
    ) -> Self {
        Self {
            composition,
            format_tree: Some(format_tree),
            defs: Vec::new(),
            source,
        }
    }
}
