
use crate::core::scanner::Scanner;
use super::errors::{ErrorKind, Error};



enum HtmlState {
    Data,
    AfterStartTagName,
}

pub enum HtmlToken<'a> {
    Text(&'a [u8]),

    StartTag(&'a [u8]),

    EndTag(&'a [u8]),

    TagEnd {
        self_closing: bool,
    },

    Comment(&'a [u8]),

    CData(&'a [u8]),

    DocType(&'a [u8]),
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
            Self::EndTag(a) => write!(f, "EndTag({})", String::from_utf8_lossy(a)),
            Self::TagEnd { self_closing } => write!(f, "TagEnd(self_closing={})", self_closing),
            Self::Comment(a) => write!(f, "Comment({})", String::from_utf8_lossy(a)),
            Self::CData(a) => write!(f, "CData({})", String::from_utf8_lossy(a)),
            Self::DocType(a) => write!(f, "DocType({})", String::from_utf8_lossy(a)),
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
                self.scanner.consume_n_bytes(4);
                match self.scanner.consume_comment() {
                    Some(comment) => return Some(Ok(HtmlToken::Comment(comment))),
                    None => return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile))),
                }
            }

            if self.scanner.starts_with(b"<![CDATA[") {
                self.scanner.consume_n_bytes(9);
                match self.scanner.consume_cdata() {
                    Some(comment) => return Some(Ok(HtmlToken::CData(comment))),
                    None => return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile))),
                }
            }

            if self.scanner.starts_with(b"<!DOCTYPE") {
                self.scanner.consume_n_bytes(9);
                if let Some(value) = self.scanner.consume_until(|b| b == b'>') {
                    self.scanner.consume_byte();
                    return Some(Ok(HtmlToken::DocType(value)));
                }
                return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));

            }
        }

        if self.scanner.starts_with(b"</") {
            self.scanner.advance_past_whitespaces();
            if self.scanner.is_eof() {
                return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
            }

            let name = match self.scanner.consume_tag_name() {
                Some(name) => name,
                None => return  Some(Err(self.error(ErrorKind::UnexpectedEndOfFile))),
            };
            
            // FIXME: actually consume attributes in the end tag.
            if self.scanner.consume_until(|b| b == b'>').is_none() {
                return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
            }
            self.scanner.consume_byte();
            return Some(Ok(HtmlToken::EndTag(name)));

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

    fn next_after_start_tag_name(&mut self) -> Option<Result<HtmlToken<'a>, Error>> {
        self.scanner.advance_past_whitespaces();
        if self.scanner.is_eof() {
            return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
        }

        if self.scanner.starts_with(b"/>") {
            self.scanner.consume_n_bytes(2);
            self.state = HtmlState::Data;

            Some(Ok(HtmlToken::TagEnd { self_closing: true }))
        } else if self.scanner.is_byte(b'>') {
            self.scanner.consume_byte();
            self.state = HtmlState::Data;
            Some(Ok(HtmlToken::TagEnd { self_closing: false }))

        } else {
            self.scanner.consume_byte();
            Some(Ok(HtmlToken::Text(b"")))
            //Some(self.consume_attribute())
        }
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
            HtmlState::AfterStartTagName => self.next_after_start_tag_name(), 
        } 
    }
}
