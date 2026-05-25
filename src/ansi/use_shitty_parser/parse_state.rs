#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParseState {
    #[default]
    Ground,
    Escape,
    EscapeCharset,
    Csi,
    Osc,
    OscEscape,
}
