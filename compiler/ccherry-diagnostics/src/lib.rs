//! Diagnostics for the Cherry compiler.

pub use codespan_reporting::diagnostic::{Diagnostic, Label, LabelStyle, Severity};
pub use codespan_reporting::term::{Chars, DisplayStyle, Styles as Colors, termcolor::{Color, ColorChoice, ColorSpec}};

use codespan_reporting::term::{Config, termcolor};
use codespan_reporting::files::SimpleFile;

/// The "theme" to use for diagnostics.
#[derive(Clone, Debug)]
pub struct DiagnosticTheme {
    /// Whether or not to use colors in this diagnostic theme.
    pub color_choice: ColorChoice,

    /// The characters for the diagnostic theme to use.
    pub chars: Chars,

    /// The display style for the diagnostic to use.
    pub display_style: DisplayStyle,

    /// The colors to use for the diagnostic theme.
    pub colors: Colors,

    /// How many spaces are in a tab character.
    pub tab_width: usize,

    /// The minimum number of lines to be shown after the line on which a multiline Label begins.
    pub start_context_lines: usize,

    /// The minimum number of lines to be shown before the line on which a multiline Label ends.
    pub end_context_lines: usize,
}

impl DiagnosticTheme {
    /// Initializes the default diagnostic theme for Cherry.
    /// 
    /// By default, diagnostics use ASCII characters and automatic colors.
    pub fn new() -> Self {
        Self {
            color_choice: ColorChoice::Auto,
            chars: Chars::ascii(),
            display_style: DisplayStyle::Rich,
            colors: Colors::default(),
            tab_width: 4,
            start_context_lines: 2,
            end_context_lines: 1,
        }
    }

    /// Returns this diagnostic theme after using the provided characters.
    pub fn with_chars(mut self, chars: Chars) -> Self {
        self.chars = chars;
        self
    }

    /// Returns this diagnostic theme after using the provided display style.
    pub fn with_display_style(mut self, display_style: DisplayStyle) -> Self {
        self.display_style = display_style;
        self
    }

    /// Returns this diagnostic theme after using the provided colors.
    pub fn with_colors(mut self, colors: Colors) -> Self {
        self.colors = colors;
        self
    }

    /// Returns this diagnostic theme after using the provided context lines.
    pub fn with_context_lines(mut self, start: usize, end: usize) -> Self {
        self.start_context_lines = start;
        self.end_context_lines = end;
        self
    }

    /// Returns the "Rustc" theme.
    pub fn rustc() -> Self {
        let mut red = ColorSpec::new();
        red.set_fg(Some(Color::Red));
        red.set_intense(true);
        red.set_bold(true);

        let mut yellow = ColorSpec::new();
        yellow.set_fg(Some(Color::Yellow));
        yellow.set_intense(true);
        yellow.set_bold(true);

        let mut blue = ColorSpec::new();
        blue.set_fg(Some(Color::Blue));
        blue.set_intense(true);
        blue.set_bold(true);

        let mut bold = ColorSpec::new();
        bold.set_intense(true);
        bold.set_bold(true);

        Self::new()
            .with_chars(Chars::ascii())
            .with_colors(Colors {
                header_bug: red.clone(),
                header_error: red.clone(),
                header_warning: yellow.clone(),
                header_note: blue.clone(),
                header_help: blue.clone(),
                header_message: bold.clone(),
                primary_label_bug: red.clone(),
                primary_label_error: red.clone(),
                primary_label_warning: yellow.clone(),
                primary_label_note: blue.clone(),
                primary_label_help: blue.clone(),
                secondary_label: blue.clone(),
                line_number: blue.clone(),
                source_border: blue.clone(),
                note_bullet: blue.clone(),
            })
    }
}

impl Default for DiagnosticTheme {
    fn default() -> Self {
        Self::new()
    }
}

impl Into<Config> for DiagnosticTheme {
    fn into(self) -> Config {
        Config {
            chars: self.chars,
            display_style: self.display_style,
            styles: self.colors,
            tab_width: self.tab_width,
            start_context_lines: self.start_context_lines,
            end_context_lines: self.end_context_lines,
        }
    }
}

/// An emitter for diagnostics, which emits diagnostics to the console.
pub struct DiagnosticEmitter {
    /// The name of the file this DiagnosticEmitter is for.
    filename: String,

    /// The contents of the source file.
    source: String,

    /// The theme for the emitter to use.
    theme: DiagnosticTheme,
}

impl DiagnosticEmitter {
    /// Creates a new [`DiagnosticEmitter`].
    pub fn new(filename: String, source: String) -> Self {
        Self {
            filename,
            source,
            theme: DiagnosticTheme::default(),
        }
    }

    /// Uses the provided theme.
    pub fn with_theme(mut self, theme: DiagnosticTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Emits a diagnostic message to the terminal.
    pub fn emit(&self, diagnostic: &Diagnostic<()>) {
        let files = SimpleFile::new(self.filename.to_string(), self.source.to_string());
        codespan_reporting::term::emit(
            &mut termcolor::BufferedStandardStream::stdout(self.theme.color_choice),
            &self.theme.clone().into(),
            &files,
            &diagnostic).unwrap();
    }

    /// Emits all diagnostics in a [`Vec`] to the terminal.
    pub fn emit_all(&self, diagnostics: &Vec<Diagnostic<()>>) {
        for diagnostic in diagnostics {
            self.emit(diagnostic);
        }
    }
}
