use crate::core::scanner::Scanner;
use super::errors::{ErrorKind, Error};

#[derive(Debug)]
enum HtmlState {
    Data,
    AfterStartTagName,
    RawText,
}

pub enum HtmlToken<'a> {
    Text(&'a [u8]),

    RawText(&'a [u8]),

    StartTag(&'a [u8]),

    EndTag(&'a [u8]),

    TagEnd {
        self_closing: bool,
    },

    Comment(&'a [u8]),

    CData(&'a [u8]),

    DocType(&'a [u8]),

    Attribute {
        name: &'a [u8],
        value: &'a [u8],
    },

    BoolAttribute {
        name: &'a [u8],
    }
}

pub struct HtmlTokenizer<'a> {
    scanner: Scanner<'a>,
    state: HtmlState,
    raw_text_tag_name: Option<&'a [u8]>,
}

impl<'a> From<&'a [u8]> for HtmlTokenizer<'a> {
    fn from(value: &'a [u8]) -> Self {
        let scanner = Scanner::from(value);
        Self {
            scanner,
            state: HtmlState::Data,
            raw_text_tag_name: None,
        }
    }
}

impl<'a> std::fmt::Display for HtmlToken<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(a) => write!(f, "Text({})", String::from_utf8_lossy(a)),
            Self::RawText(a) => write!(f, "RawText({})", String::from_utf8_lossy(a)),
            Self::StartTag(a) => write!(f, "StartTag({})", String::from_utf8_lossy(a)),
            Self::EndTag(a) => write!(f, "EndTag({})", String::from_utf8_lossy(a)),
            Self::TagEnd { self_closing } => write!(f, "TagEnd(self_closing={})", self_closing),
            Self::Comment(a) => write!(f, "Comment({})", String::from_utf8_lossy(a)),
            Self::CData(a) => write!(f, "CData({})", String::from_utf8_lossy(a)),
            Self::DocType(a) => write!(f, "DocType({})", String::from_utf8_lossy(a)),
            Self::Attribute { name, value } => write!(f, "Attribute(name={}, value={})", String::from_utf8_lossy(name), String::from_utf8_lossy(value)),
            Self::BoolAttribute{ name } => write!(f, "Attribute(name={})", String::from_utf8_lossy(name)),
        }
    }
} 

impl<'a> HtmlTokenizer<'a> {

    #[inline]
    fn error(&self, kind: ErrorKind) -> Error {
        Error { kind, pos: self.scanner.pos }
    }

    #[inline]
    fn unexpected_eof_error(&self) -> Error {
        Error { kind: ErrorKind::UnexpectedEndOfFile, pos: self.scanner.pos }
    }
    
    #[inline]
    fn consume_attribute(&mut self) -> Result<HtmlToken<'a>, Error> {
        self.scanner.advance_past_whitespaces();
        if self.scanner.is_eof() {
            return Err(self.unexpected_eof_error());
        }

        let attribute_name = self.scanner.consume_attribute_name()
            .ok_or_else(|| self.unexpected_eof_error())?;

        self.scanner.advance_past_whitespaces();
        if self.scanner.is_eof() {
            return Err(self.unexpected_eof_error());
        }

        if !self.scanner.consume_byte_if(b'=') {
            return Ok(HtmlToken::BoolAttribute { name: attribute_name });
        }

        self.scanner.advance_past_whitespaces();
        if self.scanner.is_eof() {
            return Err(self.unexpected_eof_error());
        }

        if self.scanner.get_byte() == b'\'' || self.scanner.get_byte() == b'"' {
            let quote_char = self.scanner.get_byte();
            self.scanner.consume_byte();
            let attribute_value = self.scanner.consume_quoted_attribute_value(quote_char)
                .ok_or_else(|| self.unexpected_eof_error())?;
            self.scanner.consume_byte();
            return Ok(HtmlToken::Attribute { name: attribute_name, value: attribute_value });
        }

        let attribute_value = self.scanner.consume_unquoted_attribute_value()
            .ok_or_else(|| self.unexpected_eof_error())?;
        Ok(HtmlToken::Attribute { name: attribute_name, value: attribute_value })
    }
    
    #[inline]
    fn next_data(&mut self) -> Option<Result<HtmlToken<'a>, Error>> {
        // the data state either identifies the start of an
        // element, or consumes as text.
        if !self.scanner.is_byte(b'<') {
            return Some(Ok(HtmlToken::Text(self.scanner.consume_text())));
        }

        if self.scanner.get_checked_nth_byte(1) == Some(&b'!') {
            
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

            if self.scanner.starts_with(b"<!DOCTYPE ") {
                self.scanner.consume_n_bytes(10);
                if let Some(value) = self.scanner.consume_until(|b| b == b'>') {
                    self.scanner.consume_byte();
                    return Some(Ok(HtmlToken::DocType(value)));
                }
                return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));

            }

            // If none of the above '<!' instances match, 
            // consume it as a bogus comment.
            self.scanner.consume_byte();
            if let Some(value) = self.scanner.consume_until(|b| b == b'>') {
                self.scanner.consume_byte();
                return Some(Ok(HtmlToken::Comment(value)));
            } else {
                return Some(Err(self.unexpected_eof_error()));
            }

        }

        if self.scanner.starts_with(b"</") {
            self.scanner.consume_n_bytes(2);

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
        self.scanner.consume_byte();
        let tag_name = match self.scanner.consume_tag_name() {
            Some(name) => name,
            None => return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)))
        };
        if matches!(tag_name, b"style" | b"script") {
            self.raw_text_tag_name = Some(tag_name);
        }

        self.state = HtmlState::AfterStartTagName;
        Some(Ok(HtmlToken::StartTag(tag_name)))
    }

    #[inline]
    fn next_after_start_tag_name(&mut self) -> Option<Result<HtmlToken<'a>, Error>> {
        self.scanner.advance_past_whitespaces();
        if self.scanner.is_eof() {
            return Some(Err(self.error(ErrorKind::UnexpectedEndOfFile)));
        }

        if self.scanner.starts_with(b"/>") {
            self.scanner.consume_n_bytes(2);
            
            if !self.raw_text_tag_name.is_none() {
                self.state = HtmlState::RawText;
            } else {
                self.state = HtmlState::Data;
            }

            Some(Ok(HtmlToken::TagEnd { self_closing: true }))

        } else if self.scanner.is_byte(b'>') {
            self.scanner.consume_byte();
            
            if !self.raw_text_tag_name.is_none() {
                self.state = HtmlState::RawText;
            } else {
                self.state = HtmlState::Data;
            }
            Some(Ok(HtmlToken::TagEnd { self_closing: false }))

        } else {
            Some(self.consume_attribute())
        }
    }

    #[inline]
    fn next_raw_text(&mut self) -> Option<Result<HtmlToken<'a>, Error>> {
        let name = self.raw_text_tag_name.unwrap();
        if let Some(text) = self.scanner.consume_until_end_tag(name) {
            self.state = HtmlState::Data;
            self.raw_text_tag_name = None;
            return Some(Ok(HtmlToken::RawText(text)));
        }
        Some(Err(self.unexpected_eof_error()))
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
            HtmlState::RawText => self.next_raw_text(),
        } 
    }
}
