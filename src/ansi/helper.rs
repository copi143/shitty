macro_rules! terminal_wrapper {
    () => {
        pub struct TerminalWrapper<'term> {
            terminal: &'term mut crate::Terminal,
        }

        impl core::ops::Deref for TerminalWrapper<'_> {
            type Target = crate::Terminal;

            fn deref(&self) -> &Self::Target {
                self.terminal
            }
        }

        impl core::ops::DerefMut for TerminalWrapper<'_> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.terminal
            }
        }

        impl TerminalWrapper<'_> {
            pub fn new(terminal: &'_ mut crate::Terminal) -> TerminalWrapper<'_> {
                TerminalWrapper { terminal }
            }
        }
    };
}

enum_map! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum CursorShape : usize {
        /// Cursor is a block like `▒`.
        #[default]
        Block = 0,

        /// Cursor is an underscore like `_`.
        Underline = 1,

        /// Cursor is a vertical bar `⎸`.
        Beam = 2,

        /// Cursor is a box like `☐`.
        HollowBlock = 3,

        /// Invisible cursor.
        Hidden = 4,
    }
    --- SAME AS ---
    #[cfg(feature = "vte")]
    vte::ansi::CursorShape,
}
