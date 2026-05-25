use crate::Terminal;

#[macro_use]
mod helper;
pub use helper::*;

mod use_none;

#[cfg(feature = "fast-parser")]
mod use_fast_parser;

#[cfg(feature = "vte")]
mod use_vte;

#[cfg(feature = "ansi-parser")]
mod use_ansi_parser;

#[cfg(feature = "shitty-parser")]
mod use_shitty_parser;

#[allow(clippy::large_enum_variant)]
#[allow(clippy::enum_variant_names)]
pub enum Parser {
    Invalid,
    None(use_none::Processor),
    #[cfg(feature = "fast-parser")]
    FastParser(use_fast_parser::Processor),
    #[cfg(feature = "vte")]
    Vte(use_vte::Processor),
    #[cfg(feature = "ansi-parser")]
    AnsiParser(use_ansi_parser::Processor),
    #[cfg(feature = "shitty-parser")]
    SimpleParser(use_shitty_parser::Processor),
}

impl Default for Parser {
    #[allow(unreachable_code)]
    #[allow(clippy::needless_return)]
    fn default() -> Self {
        // Parser feature precedence: vte > ansi-parser > fast-parser > shitty-parser > none.
        #[cfg(feature = "vte")]
        return Parser::Vte(use_vte::Processor::default());
        #[cfg(feature = "ansi-parser")]
        return Parser::AnsiParser(use_ansi_parser::Processor::default());
        #[cfg(feature = "fast-parser")]
        return Parser::FastParser(use_fast_parser::Processor::default());
        #[cfg(feature = "shitty-parser")]
        return Parser::SimpleParser(use_shitty_parser::Processor::default());
        return Parser::None(use_none::Processor::default());
    }
}

impl Parser {
    /// Advance the parser with the given bytes, processing them and updating the terminal state accordingly.
    pub fn advance(&mut self, terminal: &mut Terminal, bytes: &[u8]) {
        match *self {
            Parser::Invalid => panic!("Invalid parser state"),
            Parser::None(ref mut processor) => {
                processor.advance(&mut use_none::TerminalWrapper::new(terminal), bytes);
            }
            #[cfg(feature = "fast-parser")]
            Parser::FastParser(ref mut processor) => {
                processor.advance(&mut use_fast_parser::TerminalWrapper::new(terminal), bytes);
            }
            #[cfg(feature = "vte")]
            Parser::Vte(ref mut processor) => {
                processor.advance(&mut use_vte::TerminalWrapper::new(terminal), bytes);
            }
            #[cfg(feature = "ansi-parser")]
            Parser::AnsiParser(ref mut processor) => {
                processor.advance(&mut use_ansi_parser::TerminalWrapper::new(terminal), bytes);
            }
            #[cfg(feature = "shitty-parser")]
            Parser::SimpleParser(ref mut processor) => {
                processor.advance(&mut use_shitty_parser::TerminalWrapper::new(terminal), bytes);
            }
        }
    }
}
