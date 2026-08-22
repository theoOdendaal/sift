// https://www.w3.org/TR/REC-xml/

// FIXME: Update this to make it more modular. Below is a good example,
// which uses roughly the w3c html approach.
// https://github.com/servo/html5ever/blob/main/xml5ever/src/tokenizer/states.rs

use crate::xml::errors::{Error, TokenErrorKind};

#[derive(Debug)]
pub enum Token<'a> {
    Declaration(&'a str),

    CharacterData(&'a str),

    Comment(&'a str),

    StartTag(&'a str),

    Attribute { name: &'a str, value: &'a str },

    TagEnd { self_closing: bool },

    EndTag(&'a str),

    Text(&'a str),
}

enum TokenizerState {
    Normal,
    InsideTag,
    InsideComment,
    
}

pub struct Tokenizer<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
    state: TokenizerState,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
            state: TokenizerState::Normal,
        }
    }
}



impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        let remaining = &self.input[self.pos..];
        if remaining.trim().is_empty() {
            self.pos = self.input.len(); 
            return None;
        }
        
        match self.state {
            TokenizerState::Normal => {

                let start = self.pos;

                if self.bytes[start] == b'<' {
                    
                    // Check for comment.
                    if self.input[start..].starts_with("<!--") {
                        self.state = TokenizerState::InsideComment;
                        self.pos += 4;
                        return self.next();
                    }

                    // Check for declaration.
                    if self.input[start..].starts_with("<?") {
                        let declaration_end_idx = match self.input[start..].find("?>") {
                            Some(idx) => idx,
                            None => return Some(Err(Error::UnterminatedToken { pos: self.pos, kind: TokenErrorKind::Declaration })),
                        };
                        let declaration_str = &self.input[start+2..start+declaration_end_idx];
                        self.pos += declaration_end_idx + 2;
                        return Some(Ok(Token::Declaration(declaration_str)));
                    }

                    // Check for CData
                    if self.input[start..].starts_with("<![CDATA[") {
                        let cdata_end_idx = match self.input[start..].find("]]>") {
                            Some(idx) => idx,
                            None => return Some(Err(Error::UnterminatedToken { pos: self.pos, kind: TokenErrorKind::CharacterData })),
                        };
                        let cdata_str = &self.input[start+9..start+cdata_end_idx];
                        self.pos += cdata_end_idx + 3;
                        return Some(Ok(Token::CharacterData(cdata_str)));
                    }

                    // Check for end tag, for non self-closing start tag.
                    if self.input[start..].starts_with("</") {
                        self.pos += 2;
                        
                        let tag_name_end_idx = match self.input[self.pos..].find('>') {
                            Some(idx) => idx,
                            None => return Some(Err(Error::UnterminatedToken { pos: self.pos, kind: TokenErrorKind::End })),
                        };
                        let name = &self.input[self.pos..self.pos+tag_name_end_idx];
                        self.pos += tag_name_end_idx + 1;

                        return Some(Ok(Token::EndTag(name)));
                    }

                    // Start tag logic.
                    self.state = TokenizerState::InsideTag;
                    self.pos += 1;
                    let tag_name_end_idx = match self.input[self.pos..].find(|c: char| c.is_whitespace() || c == '>') {
                        Some(idx) => idx,
                        None => return Some(Err(Error::UnterminatedToken { pos: self.pos, kind: TokenErrorKind::Start })),
                    };
                    let name = &self.input[self.pos..self.pos+tag_name_end_idx];
                    self.pos += tag_name_end_idx;
                    return Some(Ok(Token::StartTag(name)));


                } else {
                    // Text logic
                    
                    // The tokenizer assumes that the next few chars are
                    // 'Text' if the state is 'Normal', and the current char
                    // is not '<'.

                    let text_end_idx = match self.input[self.pos..].find('<') {
                        Some(idx) => idx,
                        None => return Some(Err(Error::UnterminatedToken { pos: self.pos, kind: TokenErrorKind::Text })),
                    };
                    let text = &self.input[self.pos..self.pos+text_end_idx];
                    self.pos += text_end_idx;
                    
                    if text.trim().is_empty() {
                        return self.next();
                    }

                    return Some(Ok(Token::Text(text)));
                }
            },

            TokenizerState::InsideTag => {
                // Skip whitespaces.
                while self.pos < self.input.len() && self.bytes[self.pos].is_ascii_whitespace() {
                    self.pos += 1;
                }

                if self.pos >= self.input.len() {
                    return Some(Err(Error::UnterminatedToken { pos: self.pos, kind: TokenErrorKind::Tag }));
                }

                // Check for non self-closing.
                if self.bytes[self.pos] == b'>' {
                    self.pos += 1;
                    self.state = TokenizerState::Normal;
                    return Some(Ok(Token::TagEnd { self_closing: false}))
                }

                if self.input[self.pos..].starts_with("/>") {
                    self.pos += 2;
                    self.state = TokenizerState::Normal;
                    return Some(Ok(Token::TagEnd { self_closing: true }));
                }

                // Parse attribute name.
                let remaining = &self.input[self.pos..];
                let eq_idx = match remaining.find('=') {
                    Some(idx) => idx,
                    None => return Some(Err(Error::MissingExpectedChar { pos: self.pos, expected_char: '=', kind: TokenErrorKind::Attribute })),
                };
                let name = remaining[..eq_idx].trim();
                self.pos += eq_idx + 1;
                
                // Parse attribute value.
                let remaining = &self.input[self.pos..];
                let quote_char = match remaining.chars().next() {
                    Some(c @ ('\'' | '"')) => c,
                    _ => {
                        return Some(Err(Error::UnquotedToken { pos: self.pos, kind: TokenErrorKind::Attribute }))
                    },
                };
                self.pos += 1;
                let value_end_idx = match self.input[self.pos..].find(quote_char) {
                    Some(idx) => idx,
                    None => return Some(Err(Error::UnterminatedToken { pos: self.pos, kind: TokenErrorKind::Attribute })),
                };
                let value = &self.input[self.pos..self.pos+value_end_idx];
                self.pos += value_end_idx + 1;
                return Some(Ok(Token::Attribute { name, value }));

            },
            
            TokenizerState::InsideComment => {
                
                match self.input[self.pos..].find("-->") {
                    Some(comment_end_idx) => {
                        let comment_text = &self.input[self.pos..self.pos + comment_end_idx];
                        self.pos += comment_end_idx + 3;
                        self.state = TokenizerState::Normal;
                        return Some(Ok(Token::Comment(comment_text)));
                    },
                    None => return Some(Err(Error::UnterminatedToken { pos: self.pos , kind: TokenErrorKind::Comment })),
                }

            },
        }
    }
}
