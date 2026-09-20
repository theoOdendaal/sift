
use crate::core::scanner::Scanner;
use super::errors::{ErrorKind, Error};



enum HtmlState {
    Data,
    AfterStartTagName,
}

pub enum HtmlToken<'a> {
    Text(&'a [u8]),

    StartTag(&'a [u8]),
}

pub struct HtmlTokenizer<'a> {
    scanner: Scanner<'a>,
    state: HtmlState,
}

impl<'a> From<&'a [u8]> for HtmlTokenizer<'a> {
    fn from(value: &'a [u8]) -> Self {
        let scanner = Scanner::from(value);
        Self { scanner, state: HtmlState::Data }
    }
}

impl<'a> std::fmt::Display for HtmlToken<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(a) => write!(f, "Text({})", String::from_utf8_lossy(a)),
            Self::StartTag(a) => write!(f, "StartTag({})", String::from_utf8_lossy(a)),
        }
    }
} 

impl<'a> HtmlTokenizer<'a> {

    #[inline]
    fn error(&self, kind: ErrorKind) -> Error {
        Error { kind, pos: self.scanner.pos }
    }
    
    #[inline]
    fn next_data(&mut self) -> Option<Result<HtmlToken<'a>, Error>> {
        // the data either identifies the start of an
        // element, or consumes as text.
        if !self.scanner.is_byte(b'<') {
            return Some(Ok(HtmlToken::Text(self.scanner.consume_text())));
        }
        self.scanner.consume_byte();

        if self.scanner.is_byte(b'!') {
            if self.scanner.starts_with(b"<!--") {

            }

            if self.scanner.starts_with(b"<![CDATA[") {

            }

            if self.scanner.starts_with(b"<!DOCTYPE") {

            }
        }


        if self.scanner.starts_with(b"</") {

        }

        // Consume as tag name if no other patterns
        // matched.
        let tag_name = match self.scanner.consume_tag_name() {
            Some(name) => name,
            None => return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)))
        };
        self.state = HtmlState::AfterStartTagName;
        Some(Ok(HtmlToken::StartTag(tag_name)))
    }
}

impl<'a> Iterator for HtmlTokenizer<'a> {
    type Item = Result<HtmlToken<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.scanner.is_eof() {
            return None;
        }

        match self.state {
            HtmlState::Data => self.next_data(),
            HtmlState::AfterStartTagName => unimplemented!("AfterStartTagName")
        } 
    }
}
