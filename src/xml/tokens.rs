// https://www.w3.org/TR/REC-xml/

use crate::xml::errors::{Error, TokenErrorKind};

#[derive(Debug)]
pub enum XmlToken<'a> {
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

pub struct XmlTokenizer<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
    state: TokenizerState,
}

impl<'a> XmlTokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
            state: TokenizerState::Normal,
        }
    }
}

impl<'a> Iterator for XmlTokenizer<'a> {
    type Item = Result<XmlToken<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        match self.state {
            TokenizerState::Normal => {
                let start = self.pos;

                if self.bytes[start] == b'<' {

                    let remaining = &self.input[start..];

                    // Check for comment.
                    if remaining.starts_with("<!--") {
                        self.state = TokenizerState::InsideComment;
                        self.pos += 4;
                        return self.next();
                    }

                    // Check for declaration.
                    if remaining.starts_with("<?") {
                        let declaration_end_idx = match self.input[start..].find("?>") {
                            Some(idx) => idx,
                            None => {
                                return Some(Err(Error::UnterminatedToken {
                                    pos: self.pos,
                                    kind: TokenErrorKind::Declaration,
                                }));
                            }
                        };
                        let declaration_str = &self.input[start + 2..start + declaration_end_idx];
                        self.pos += declaration_end_idx + 2;
                        return Some(Ok(XmlToken::Declaration(declaration_str)));
                    }

                    // Check for DOCTYPE
                    /*if remaining.starts_with("<!DOCTYPE") {
                         
                    }*/

                    // Check for CData
                    if remaining.starts_with("<![CDATA[") {
                        let cdata_end_idx = match self.input[start..].find("]]>") {
                            Some(idx) => idx,
                            None => {
                                return Some(Err(Error::UnterminatedToken {
                                    pos: self.pos,
                                    kind: TokenErrorKind::CharacterData,
                                }));
                            }
                        };
                        let cdata_str = &self.input[start + 9..start + cdata_end_idx];
                        self.pos += cdata_end_idx + 3;
                        return Some(Ok(XmlToken::CharacterData(cdata_str)));
                    }

                    // Check for end tag, for non self-closing start tag.
                    if remaining.starts_with("</") {
                        self.pos += 2;

                        let tag_name_end_idx = match self.input[self.pos..].find('>') {
                            Some(idx) => idx,
                            None => {
                                return Some(Err(Error::UnterminatedToken {
                                    pos: self.pos,
                                    kind: TokenErrorKind::End,
                                }));
                            }
                        };
                        let name = &self.input[self.pos..self.pos + tag_name_end_idx].trim();
                        self.pos += tag_name_end_idx + 1;

                        return Some(Ok(XmlToken::EndTag(name)));
                    }

                    // Start tag logic.
                    self.state = TokenizerState::InsideTag;
                    self.pos += 1;
                    let tag_name_end_idx = match self.input[self.pos..]
                        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                    {
                        Some(idx) => idx,
                        None => {
                            return Some(Err(Error::UnterminatedToken {
                                pos: self.pos,
                                kind: TokenErrorKind::Start,
                            }));
                        }
                    };
                    let name = &self.input[self.pos..self.pos + tag_name_end_idx];
                    self.pos += tag_name_end_idx;
                    Some(Ok(XmlToken::StartTag(name)))
                } else {
                    // Text logic
                    
                    match self.input[self.pos..].find('<') {
                        Some(text_end_idx) => {
                            let text = &self.input[self.pos..self.pos + text_end_idx];
                            self.pos += text_end_idx;

                            if text.trim().is_empty() {
                                return self.next();
                            } else {
                                return Some(Ok(XmlToken::Text(text)));
                            }
                        },
                        None => {
                            let rest = &self.input[self.pos..];
                            if rest.trim().is_empty() {
                                self.pos = self.input.len();
                                return None;
                            } else {
                                return Some(Err(Error::UnterminatedToken {
                                    pos: self.pos,
                                    kind: TokenErrorKind::Text,
                                }))

                            }
                        }
                    }
                }
            },

            TokenizerState::InsideTag => {
                // Skip whitespaces.
                while self.pos < self.input.len() && self.bytes[self.pos].is_ascii_whitespace() {
                    self.pos += 1;
                }

                if self.pos >= self.input.len() {
                    return Some(Err(Error::UnterminatedToken {
                        pos: self.pos,
                        kind: TokenErrorKind::Tag,
                    }));
                }

                // Check for non self-closing.
                if self.bytes[self.pos] == b'>' {
                    self.pos += 1;
                    self.state = TokenizerState::Normal;
                    return Some(Ok(XmlToken::TagEnd {
                        self_closing: false,
                    }));
                }

                if self.input[self.pos..].starts_with("/>") {
                    self.pos += 2;
                    self.state = TokenizerState::Normal;
                    return Some(Ok(XmlToken::TagEnd { self_closing: true }));
                }

                // Parse attribute name.
                let remaining = &self.input[self.pos..];
                let gt_idx = remaining.find('>');
                let eq_idx = match remaining.find('=') {
                    Some(eq) if gt_idx.map_or(true, | gt| eq < gt)=> eq,
                    _ => {
                        return Some(Err(Error::MissingExpectedChar {
                            pos: self.pos,
                            expected_char: '=',
                            kind: TokenErrorKind::Attribute,
                        }));
                    }
                };
                let name = remaining[..eq_idx].trim();
                self.pos += eq_idx + 1;

                // Parse attribute value.
                let remaining = &self.input[self.pos..];
                let quote_char = match remaining.chars().next() {
                    Some(c @ ('\'' | '"')) => c,
                    _ => {
                        return Some(Err(Error::UnquotedToken {
                            pos: self.pos,
                            kind: TokenErrorKind::Attribute,
                        }));
                    }
                };
                self.pos += 1;

                let remaining = &self.input[self.pos..];
                let lt_idx = remaining.find('<');
                let value_end_idx = match self.input[self.pos..].find(quote_char) {
                    Some(q) if lt_idx.map_or(true, |lt| lt > q) => q,
                    _ => {
                        return Some(Err(Error::UnterminatedToken {
                            pos: self.pos,
                            kind: TokenErrorKind::Attribute,
                        }));
                    }
                };
                let value = &self.input[self.pos..self.pos + value_end_idx];
                self.pos += value_end_idx + 1;
                Some(Ok(XmlToken::Attribute { name, value }))
            }

            TokenizerState::InsideComment => match self.input[self.pos..].find("-->") {
                Some(comment_end_idx) => {
                    let comment_text = &self.input[self.pos..self.pos + comment_end_idx];
                    self.pos += comment_end_idx + 3;
                    self.state = TokenizerState::Normal;
                    Some(Ok(XmlToken::Comment(comment_text)))
                }
                None => Some(Err(Error::UnterminatedToken {
                    pos: self.pos,
                    kind: TokenErrorKind::Comment,
                })),
            },

        }
    }
}
