use std::process::exit;

use clap::{Arg, Command};
use ccherry_diagnostics::{Diagnostic, DiagnosticTheme, DiagnosticEmitter, DisplayStyle};
use ccherry_lexer::Lexer;

/// Configuration for the Cherry command line.
pub struct CherryConfig {
    /// The path to the file to compile.
    input: String,

    /// The diagnostic style to use.
    diagnostic_style: DisplayStyle,

    /// The diagnostic theme to use.
    theme: DiagnosticTheme,
}

impl CherryConfig {
    pub fn parse() -> Self {
        let args = Command::new("ccherry")
            .about("the Cherry compiler")
            .bin_name("ccherry")
            .arg(Arg::new("input")
                .index(1)
                .takes_value(true)
                .required(true)
                .help("the input file to compile"))
            .arg(Arg::new("diagnostic-style")
                .takes_value(true)
                .required(false)
                .long("diagnostic-style")
                .alias("d-style")
                .alias("diag-style")
                .alias("diagstyle")
                .alias("display-style")
                .alias("displaystyle")
                .help("what diagnostic style to use (rich, medium, short)"))
            .arg(Arg::new("theme")
                .takes_value(true)
                .required(false)
                .long("theme")
                .alias("diagnostic-theme")
                .alias("d-theme")
                .help("the diagnostic theme to use."))
            .get_matches();
        
        let input = args.value_of("input").unwrap();

        let mut diagnostic_style = DisplayStyle::Rich;
        if let Some(display_style) = args.value_of("diagnostic-style") {
            match display_style.to_lowercase().as_str() {
                "rich" | "default" => diagnostic_style = DisplayStyle::Rich,
                "medium" => diagnostic_style = DisplayStyle::Medium,
                "short" => diagnostic_style = DisplayStyle::Short,
                _ => {
                    let emitter = DiagnosticEmitter::new("".into(), "".into());
                    emitter.emit(&Diagnostic::error()
                        .with_message("invalid diagnostic style, options: rich/default, medium, short"));
                }
            }
        }

        let mut theme = DiagnosticTheme::default();
        if let Some(display_style) = args.value_of("theme") {
            match display_style.to_lowercase().as_str() {
                "default" => {},
                "rustc" => theme = DiagnosticTheme::rustc(),
                _ => {
                    let emitter = DiagnosticEmitter::new("".into(), "".into());
                    emitter.emit(&Diagnostic::error()
                        .with_message("invalid diagnostic theme, options: default, rustc"));
                }
            }
        }

        Self {
            input: input.into(),
            diagnostic_style,
            theme,
        }
    }
}

fn main() {
    let args = CherryConfig::parse();

    let mut theme = args.theme;
    theme.display_style = args.diagnostic_style;

    match std::fs::read_to_string(args.input.clone()) {
        Ok(str) => {
            let lexer = Lexer::new(&str.clone());

            for token in lexer {
                match token {
                    Ok(token) => println!("{:#?}", token),
                    Err(diagnostic) => {
                        let emitter = DiagnosticEmitter::new(args.input, str)
                            .with_theme(theme);
                        emitter.emit(&diagnostic);
                        exit(1);
                    }
                }
            }
        },
        Err(_) => {
            let emitter = DiagnosticEmitter::new("".into(), "".into())
                .with_theme(theme);
            emitter.emit(&Diagnostic::error()
                .with_message("unable to open input file"));
            exit(1);
        }
    }
}