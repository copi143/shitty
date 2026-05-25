terminal_wrapper!();

#[derive(Debug, Default)]
pub struct Processor {
    utf8_byte1: u8,
    utf8_byte2: u8,
    utf8_byte3: u8,
    utf8_len: u8,
}

impl Processor {
    fn send(&mut self, terminal: &mut TerminalWrapper<'_>, c: char) {
        match c {
            '\t' => terminal.tab(),
            '\r' => terminal.cr(),
            '\n' => terminal.lf(),
            _ => terminal.put(c),
        };
    }

    fn utf8_expected_len(first_byte: u8) -> u8 {
        if first_byte & 0b1000_0000 == 0 {
            1
        } else if first_byte & 0b1110_0000 == 0b1100_0000 {
            2
        } else if first_byte & 0b1111_0000 == 0b1110_0000 {
            3
        } else if first_byte & 0b1111_1000 == 0b1111_0000 {
            4
        } else {
            0
        }
    }

    fn utf8_merge(&self, now: u8, expected_len: u8) -> char {
        let code_point = match expected_len {
            2 => ((self.utf8_byte1 as u32 & 0b0001_1111) << 6) | (now as u32 & 0b0011_1111),
            3 => {
                ((self.utf8_byte1 as u32 & 0b0000_1111) << 12)
                    | ((self.utf8_byte2 as u32 & 0b0011_1111) << 6)
                    | (now as u32 & 0b0011_1111)
            }
            4 => {
                ((self.utf8_byte1 as u32 & 0b0000_0111) << 18)
                    | ((self.utf8_byte2 as u32 & 0b0011_1111) << 12)
                    | ((self.utf8_byte3 as u32 & 0b0011_1111) << 6)
                    | (now as u32 & 0b0011_1111)
            }
            _ => unreachable!(),
        };
        core::char::from_u32(code_point).unwrap_or('�')
    }

    fn next_byte(&mut self, terminal: &mut TerminalWrapper<'_>, byte: u8, expected_len: u8) {
        match self.utf8_len {
            1 if expected_len == 2 => self.send(terminal, self.utf8_merge(byte, expected_len)),
            1 => self.utf8_byte1 = byte,
            2 if expected_len == 3 => self.send(terminal, self.utf8_merge(byte, expected_len)),
            2 => self.utf8_byte3 = byte,
            3 => self.send(terminal, self.utf8_merge(byte, expected_len)),
            _ => unreachable!(),
        }
    }

    pub fn advance(&mut self, terminal: &mut TerminalWrapper<'_>, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        let mut i = 0;
        let expected_len = Self::utf8_expected_len(bytes[i]);
        while self.utf8_len > 0 && i < bytes.len() {
            let byte = bytes[i];
            if byte & 0b1100_0000 == 0b1000_0000 {
                self.next_byte(terminal, byte, expected_len);
                i += 1;
            } else {
                self.send(terminal, '�');
                self.utf8_len = 0;
            }
        }
        while i < bytes.len() {
            let byte = bytes[i];
            i += 1;
            let expected_len = Self::utf8_expected_len(byte);
            if expected_len == 0 {
                self.send(terminal, '�');
            } else if expected_len == 1 {
                self.send(terminal, byte as char);
            } else {
                self.utf8_byte1 = byte;
                self.utf8_len = expected_len;
                while self.utf8_len > 0 && i < bytes.len() {
                    let byte = bytes[i];
                    if byte & 0b1100_0000 == 0b1000_0000 {
                        self.next_byte(terminal, byte, expected_len);
                        i += 1;
                    } else {
                        self.send(terminal, '�');
                        self.utf8_len = 0;
                    }
                }
            }
        }
    }
}
