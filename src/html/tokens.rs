// FIXME: I'm never updating the last_start_tag_name value.

// FIXME: Any option value should be set back to None once emitted. i.e. current_tag_buffer etc.

// FIXME: I need to work and fix the character reference model. I don't want to emit them
// as separate tokens. If they form part of an attribute name, then the name will be
// broken into various pieces. Should I perhaps resolve this in the AST?

// https://html.spec.whatwg.org/#tokenization

/*
   -- Design overview

   Multiple characters are not accumuldated using a string buffer,
   but rather by setting the mark index equal to the current index,
   and advancing the current index.

   The mark index should be normally updated in the state
   immediately preceding the state in which the multiple characters
   are accumulated and emitted.

   In any state where multiple characters need to be accumulated,
   a loop {} is used.

   For those states which have a "After*" state, the token must be
   emitted for this state rather than the accumulation state.

   If in any state the next input character needs to be reconsumed
   in another state, the 'current input character' should be evaluated
   using the peek function. Remember to manually consume the character
   for all other branches in this instance.


*/

use std::borrow::Cow;

use crate::html::errors::Error;

#[derive(Debug, PartialEq)]
pub struct Doctype<'a> {
    pub name: Option<Cow<'a, str>>,
    pub public_identifier: Option<Cow<'a, str>>,
    pub system_identifier: Option<Cow<'a, str>>,
    pub force_quirks_flag: bool,
}

#[derive(Debug, PartialEq)]
pub struct Attribute<'a> {
    pub name: Option<Cow<'a, str>>,
    pub value: Option<Cow<'a, str>>,
}

#[derive(Debug, PartialEq)]
pub struct Tag<'a> {
    pub name: Option<Cow<'a, str>>,
    pub self_closing_tag: Option<bool>,
    pub attributes: Vec<Attribute<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum HtmlToken<'a> {
    Doctype(Doctype<'a>),

    StartTag(Tag<'a>),

    EndTag(Tag<'a>),

    Comment(Cow<'a, str>),

    Character(Cow<'a, str>),

    EndOfFile,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenizationState {
    Data,

    RcData,

    RawText,

    ScriptData,

    PlainText,

    TagOpen,

    EndTagOpen,

    TagName,

    RcDataLessThanSign,

    RcDataEndTagOpen,

    RcDataEndTagName,

    RawTextLessThanSign,

    RawTextEndTagOpen,

    RawTextEndTagName,

    ScriptDataLessThanSign,

    ScriptDataEndTagOpen,

    ScriptDataEndTagName,

    ScriptDataEscapeStart,

    ScriptDataEscapeStartDash,

    ScriptDataEscaped,

    ScriptDataEscapedDash,

    ScriptDataEscapedDashDash,

    ScriptDataEscapedLessThanSign,

    ScriptDataEscapedEndTagOpen,

    ScriptDataEscapedEndTagName,

    ScriptDataDoubleEscapeStart,

    ScriptDataDoubleEscaped,

    ScriptDataDoubleEscapedDash,

    ScriptDataDoubleEscapedDashDash,

    ScriptDataDoubleEscapedLessThanSign,

    ScriptDataDoubleEscapeEnd,

    BeforeAttributeName,

    AttributeName,

    AfterAttributeName,

    BeforeAttributeValue,

    AttributeValueDoubleQuoted,

    AttributeValueSingleQuoted,

    AttributeValueUnquoted,

    AfterAttributeValueQuoted,

    SelfClosingStartTag,

    BogusComment,

    MarkupDeclarationOpen,

    CommentStart,

    CommentStartDash,

    Comment,

    CommentLessThanSign,

    CommentLessThanSignBang,

    CommentLessThanSignBangDash,

    CommentLessThanSignBangDashDash,

    CommentEndDash,

    CommentEnd,

    CommentEndBang,

    Doctype,

    BeforeDoctypeName,

    DoctypeName,

    AfterDoctypeName,

    AfterDoctypePublicKeyword,

    BeforeDoctypePublicIdentifier,

    DoctypePublicIdentifierDoubleQuoted,

    DoctypePublicIdentifierSingleQuoted,

    AfterDoctypePublicIdentifier,

    BetweenDoctypePublicAndSystemIdentifiers,

    AfterDoctypeSystemKeyword,

    BeforeDoctypeSystemIdentifier,

    DoctypeSystemIdentifierDoubleQuoted,

    DoctypeSystemIdentifierSingleQuoted,

    AfterDoctypeSystemIdentifier,

    BogusDoctype,

    CdataSection,

    CdataSectionBracket,

    CdataSectionEnd,

    ProcessingInstructionOpen,

    ProcessingInstructionTarget,

    AfterProcessingInstructionTarget,

    ProcessingInstructionData,

    ProcessingInstructionQuestionable,

    CharacterReference,

    NamedCharacterReference,

    AmbiguousAmpersand,

    NumericCharacterReference,

    HexadecimalCharacterReferenceStart,

    HexadecimalCharacterReference,

    DecimalCharacterReference,

    NumericCharacterReferenceEnd,
}

#[derive(Debug)]
pub struct HtmlTokenizer<'a> {
    input: &'a str,

    pos: usize,

    state: TokenizationState,

    return_state: TokenizationState,

    errors: Vec<Error>,

    mark: usize,

    current_tag_buffer: Option<Tag<'a>>,

    is_current_tag_end: bool,

    last_start_tag_name: Option<Cow<'a, str>>,

    current_doctype_buffer: Option<Doctype<'a>>,
    //known_next_token: Option<HtmlToken<'a>>,
}

impl<'a> HtmlTokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            state: TokenizationState::Data,
            return_state: TokenizationState::Data,
            errors: Vec::new(),
            mark: 0,
            current_tag_buffer: None,
            is_current_tag_end: false,
            last_start_tag_name: None,
            current_doctype_buffer: None,
            //known_next_token: None,
        }
    }

    pub fn set_state(&mut self, state: TokenizationState) {
        self.state = state;
    }

    pub fn set_last_start_tag_name(&mut self, last_start_tag_name: &'a str) {
        self.last_start_tag_name = Some(Cow::Borrowed(last_start_tag_name));
    }

    #[inline]
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    #[inline]
    fn consume(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    #[inline]
    fn is_ascii_alpha(c: char) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_uppercase()
    }

    #[inline]
    fn is_appropriate_end_tag(&self) -> bool {
        if let Some(last_start) = &self.last_start_tag_name {
            let current_name = &self.input[self.mark..self.pos];
            return current_name.eq_ignore_ascii_case(last_start);
        }
        false
    }

    #[inline]
    fn to_lower_cow(slice: &'a str) -> Cow<'a, str> {
        if slice.bytes().any(|b| b.is_ascii_uppercase()) {
            Cow::Owned(slice.to_ascii_lowercase())
        } else {
            Cow::Borrowed(slice)
        }
    }

    pub fn next_token(&mut self) -> Option<HtmlToken<'a>> {
        loop {
            if self.pos >= self.input.len() && self.state == TokenizationState::Data {
                return Some(HtmlToken::EndOfFile);
            }

            match self.state {

                // FIXME: I don't like the below approach. Refactor to
                // use logic similar to ScriptData.
                // https://html.spec.whatwg.org/#data-state
                TokenizationState::Data => {
                    self.mark = self.pos;

                    // 'Anything else' logic. Will consume until characters
                    // until one is encountered that change the state.
                    while let Some(c) = self.peek() {
                        if c == '&' || c == '<' || c == '\0' {
                            break;
                        }
                        self.consume();
                    }

                    if self.pos > self.mark {
                        return Some(HtmlToken::Character(Cow::Borrowed(
                            &self.input[self.mark..self.pos],
                        )));
                    }

                    match self.consume() {
                        Some('&') => {
                            self.return_state = TokenizationState::Data;
                            self.state = TokenizationState::CharacterReference;
                        }

                        Some('<') => {
                            self.state = TokenizationState::TagOpen;
                        }
                        Some('\0') => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            return Some(HtmlToken::Character(Cow::Borrowed(
                                &self.input[self.pos - 1..self.pos],
                            )));
                        }
                        None => {
                            return Some(HtmlToken::EndOfFile);
                        }
                        _ => unreachable!(),
                    }
                }

                // FIXME: Refactor similarly to ScriptData, I don't
                // like the below approach.
                // https://html.spec.whatwg.org/#rcdata-state
                TokenizationState::RcData => {
                    self.mark = self.pos;

                    // 'Anything else' logic. Will consume until characters
                    // until one is encountered that change the state.
                    while let Some(c) = self.peek() {
                        if c == '&' || c == '<' || c == '\0' {
                            break;
                        }
                        self.consume();
                    }

                    if self.pos > self.mark {
                        return Some(HtmlToken::Character(Cow::Borrowed(
                            &self.input[self.mark..self.pos],
                        )));
                    }

                    match self.consume() {
                        Some('&') => {
                            self.return_state = TokenizationState::RcData;
                            self.state = TokenizationState::CharacterReference;
                        }
                        Some('<') => {
                            self.state = TokenizationState::RcDataLessThanSign;
                        }
                        Some('\0') => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                        }
                        None => {
                            return Some(HtmlToken::EndOfFile);
                        }
                        _ => unreachable!(),
                    }
                }

                // https://html.spec.whatwg.org/#rawtext-state
                TokenizationState::RawText => {
                    self.mark = self.pos;

                    // 'Anything else' logic. Will consume until characters
                    // until one is encountered that change the state.
                    while let Some(c) = self.peek() {
                        if c == '<' || c == '\0' {
                            break;
                        }
                        self.consume();
                    }

                    if self.pos > self.mark {
                        return Some(HtmlToken::Character(Cow::Borrowed(
                            &self.input[self.mark..self.pos],
                        )));
                    }

                    match self.consume() {
                        Some('<') => {
                            self.state = TokenizationState::RawTextLessThanSign;
                        }
                        Some('\0') => {
                            // FIXME: I don't really love this, as it will results
                            // in multiple sequential Character tokens being
                            // emitted.
                            self.errors.push(Error::UnexpectedNullCharacter);
                            return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                        }
                        None => {
                            return Some(HtmlToken::EndOfFile);
                        }
                        _ => unreachable!(),
                    }
                }

                // https://html.spec.whatwg.org/#script-data-state
                TokenizationState::ScriptData => {
                    loop {
                        match self.peek() {
                            Some('<') => {
                                self.mark = self.pos;
                                self.consume();
                                self.state = TokenizationState::ScriptDataLessThanSign;
                                break;
                            },
                            Some('\0') => {
                                self.errors.push(Error::UnexpectedNullCharacter);
                                self.consume();
                            },
                            None => {
                                let slice = &self.input[self.mark..self.pos];
                                self.consume();
                                self.state = TokenizationState::Data;
                                return Some(HtmlToken::Character(Cow::Owned(slice.replace('\0', "\u{FFFD}"))));
                            },
                            _ => {
                                self.consume();
                            }
                        }
                    }
                }
                
                // FIXME: I don't like the below approach.
                // Wrap all the logic in a loop. Refactor similarly to ScriptData.
                // https://html.spec.whatwg.org/#plaintext-state
                TokenizationState::PlainText => {
                    self.mark = self.pos;

                    while let Some(c) = self.peek() {
                        if c == '\0' {
                            break;
                        }
                        self.consume();
                    }

                    if self.pos > self.mark {
                        return Some(HtmlToken::Character(Cow::Borrowed(
                            &self.input[self.mark..self.pos],
                        )));
                    }

                    match self.consume() {
                        Some('\0') => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            return Some(HtmlToken::Character(Cow::Borrowed("\u{FFFD}")));
                        }
                        None => {
                            return Some(HtmlToken::EndOfFile);
                        }
                        _ => unreachable!(),
                    }
                }

                // https://html.spec.whatwg.org/#tag-open-state
                TokenizationState::TagOpen => {
                    match self.peek() {
                        Some('!') => {
                            self.consume();
                            self.state = TokenizationState::MarkupDeclarationOpen;
                        }
                        Some('/') => {
                            self.consume();
                            self.state = TokenizationState::EndTagOpen;
                        }
                        Some(c) if Self::is_ascii_alpha(c) => {
                            //self.mark = self.pos;
                            self.current_tag_buffer = Some(Tag {
                                name: None,
                                self_closing_tag: None,
                                attributes: Vec::new(),
                            });
                            self.mark = self.pos;
                            self.state = TokenizationState::TagName;
                        }
                        Some('?') => {
                            self.consume();
                            self.state = TokenizationState::ProcessingInstructionOpen;
                        }
                        None => {
                            self.errors.push(Error::EofBeforeTagName);
                            return Some(HtmlToken::EndOfFile);
                        }
                        Some(_) => {
                            self.errors.push(Error::InvalidFirstCharacterOfTagName);
                            self.state = TokenizationState::Data;
                            return Some(HtmlToken::Character(Cow::Borrowed("<")));
                        }
                    }
                }

                // https://html.spec.whatwg.org/#end-tag-open-state
                TokenizationState::EndTagOpen => {
                    match self.peek() {
                        Some(c) if Self::is_ascii_alpha(c) => {
                            self.current_tag_buffer = Some(Tag {
                                name: None,
                                self_closing_tag: None,
                                attributes: Vec::new(),
                            });
                            self.mark = self.pos;
                            self.is_current_tag_end = true;
                            self.state = TokenizationState::TagName;
                        }
                        Some('>') => {
                            self.consume();
                            self.errors.push(Error::MissingEndTagName);
                            self.state = TokenizationState::Data;
                        }
                        None => {
                            self.errors.push(Error::EofBeforeTagName);
                            //return Some(HtmlToken::Character(Cow::Borrowed("</")));
                            return Some(HtmlToken::EndOfFile);
                        }
                        _ => {
                            self.errors.push(Error::InvalidFirstCharacterOfTagName);
                            self.state = TokenizationState::BogusComment;
                        }
                    }
                }

                // https://html.spec.whatwg.org/#tag-name-state
                TokenizationState::TagName => {
                    loop {
                        match self.peek() {
                            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                                if let Some(tag) = self.current_tag_buffer.as_mut() && tag.name.is_none() {
                                    let name_slice = &self.input[self.mark..self.pos];
                                    tag.name = Some(Self::to_lower_cow(name_slice));
                                }
                                self.consume();
                                self.state = TokenizationState::BeforeAttributeName;
                                break;
                            }
                            Some('/') => {
                                self.consume();
                                self.state = TokenizationState::SelfClosingStartTag;
                                break;
                            }
                            Some('>') => {
                                self.state = TokenizationState::Data;

                                if let Some(tag) = self.current_tag_buffer.as_mut() && tag.name.is_none(){
                                    let name_slice = &self.input[self.mark..self.pos];
                                    tag.name = Some(Self::to_lower_cow(name_slice));
                                }
                                // Onlu consime after extracting name to ensure '>'
                                // is not included as part of the name.
                                self.consume();
                                if self.is_current_tag_end {
                                    self.is_current_tag_end = false;
                                    return Some(HtmlToken::EndTag(
                                        self.current_tag_buffer.take().unwrap(),
                                    ));
                                } else {
                                    return Some(HtmlToken::StartTag(
                                        self.current_tag_buffer.take().unwrap(),
                                    ));
                                }
                            }
                            Some('\0') => {
                                self.consume();
                                self.errors.push(Error::UnexpectedNullCharacter);
                            }
                            None => {
                                self.consume();
                                self.errors.push(Error::EofInTag);
                            }
                            _ => {
                                self.consume();
                            }
                        }
                    }
                }

                // https://html.spec.whatwg.org/#rcdata-less-than-sign-state
                TokenizationState::RcDataLessThanSign => match self.peek() {
                    Some('/') => {
                        self.consume();
                        self.state = TokenizationState::RcDataEndTagOpen;
                    }
                    _ => {
                        self.state = TokenizationState::RcData;
                        return Some(HtmlToken::Character(Cow::Borrowed("<")));
                    }
                },

                // https://html.spec.whatwg.org/#rcdata-end-tag-open-state
                TokenizationState::RcDataEndTagOpen => match self.peek() {
                    Some(c) if Self::is_ascii_alpha(c) => {
                        self.current_tag_buffer = Some(Tag {
                            name: None,
                            self_closing_tag: None,
                            attributes: Vec::new(),
                        });
                        self.is_current_tag_end = true;
                        self.mark = self.pos;
                        self.state = TokenizationState::RcDataEndTagName;
                    }
                    _ => {
                        self.state = TokenizationState::RcData;
                        return Some(HtmlToken::Character(Cow::Borrowed("</")));
                    }
                },

                // https://html.spec.whatwg.org/#rcdata-end-tag-name-state
                TokenizationState::RcDataEndTagName => {
                    loop {
                        match self.peek() {
                            Some(c) if Self::is_ascii_alpha(c) => {
                                // We'll distinguish between upper and lower when emitting.
                                self.consume();
                            }

                            Some('\t') | Some('\n') | Some('\x0C') | Some(' ')
                                if self.is_appropriate_end_tag() =>
                            {
                                self.consume();
                                self.state = TokenizationState::BeforeAttributeName;
                                break;
                            }
                            Some('/') if self.is_appropriate_end_tag() => {
                                self.consume();
                                self.state = TokenizationState::SelfClosingStartTag;
                                break;
                            }
                            Some('>') if self.is_appropriate_end_tag() => {
                                self.state = TokenizationState::Data;

                                if let Some(tag) = self.current_tag_buffer.as_mut() && tag.name.is_none(){
                                    let name_slice = &self.input[self.mark..self.pos];
                                    tag.name = Some(Self::to_lower_cow(name_slice));
                                }
                                // Onlu consime after extracting name to ensure '>'
                                // is not included as part of the name.
                                self.consume();
                                return Some(HtmlToken::EndTag(
                                    self.current_tag_buffer.take().unwrap(),
                                ));
                            }
                            _ => {
                                self.current_tag_buffer = None;
                                self.state = TokenizationState::RcData;
                                // The only way to get to this state is by the preceding 2 chars
                                // being '</'.
                                let char_slice = &self.input[self.mark - 2..self.pos];
                                return Some(HtmlToken::Character(Cow::Borrowed(char_slice)));
                            }
                        }
                    }
                }

                // https://html.spec.whatwg.org/#rawtext-less-than-sign-state
                TokenizationState::RawTextLessThanSign => match self.peek() {
                    Some('/') => {
                        self.consume();
                        self.state = TokenizationState::RawTextEndTagOpen;
                    }
                    _ => {
                        self.state = TokenizationState::RawText;
                        return Some(HtmlToken::Character(Cow::Borrowed("<")));
                    }
                },

                // https://html.spec.whatwg.org/#rawtext-end-tag-open-state
                TokenizationState::RawTextEndTagOpen => match self.peek() {
                    Some(c) if Self::is_ascii_alpha(c) => {
                        self.current_tag_buffer = Some(Tag {
                            name: None,
                            self_closing_tag: None,
                            attributes: Vec::new(),
                        });
                        self.is_current_tag_end = true;
                        self.mark = self.pos;
                        self.state = TokenizationState::RawTextEndTagName;
                    }
                    _ => {
                        self.state = TokenizationState::RawText;
                        return Some(HtmlToken::Character(Cow::Borrowed("</")));
                    }
                },

                // https://html.spec.whatwg.org/#rawtext-end-tag-name-state
                TokenizationState::RawTextEndTagName => {
                    loop {
                        match self.peek() {
                            Some(c) if Self::is_ascii_alpha(c) => {
                                // We'll distinguish between upper and lower when emitting.
                                self.consume();
                            }

                            Some('\t') | Some('\n') | Some('\x0C') | Some(' ')
                                if self.is_appropriate_end_tag() =>
                            {
                                self.consume();
                                self.state = TokenizationState::BeforeAttributeName;
                                break;
                            }
                            Some('/') if self.is_appropriate_end_tag() => {
                                self.consume();
                                self.state = TokenizationState::SelfClosingStartTag;
                                break;
                            }
                            Some('>') if self.is_appropriate_end_tag() => {
                                self.state = TokenizationState::Data;

                                if let Some(tag) = self.current_tag_buffer.as_mut() && tag.name.is_none() {
                                    let name_slice = &self.input[self.mark..self.pos];
                                    tag.name = Some(Self::to_lower_cow(name_slice));
                                }
                                // Onlu consime after extracting name to ensure '>'
                                // is not included as part of the name.
                                self.consume();
                                return Some(HtmlToken::EndTag(
                                    self.current_tag_buffer.take().unwrap(),
                                ));
                            }
                            _ => {
                                self.current_tag_buffer = None;
                                self.state = TokenizationState::RawText;
                                // The only way to get to this state is by the preceding 2 chars
                                // being '</'.
                                let char_slice = &self.input[self.mark - 2..self.pos];
                                return Some(HtmlToken::Character(Cow::Borrowed(char_slice)));
                            }
                        }
                    }
                },

                // https://html.spec.whatwg.org/#script-data-less-than-sign-state
                TokenizationState::ScriptDataLessThanSign => {
                    match self.peek() {
                        Some('/') => {
                            self.consume();
                            self.state = TokenizationState::ScriptDataEndTagOpen;
                            self.mark = self.pos;
                        },
                        Some('!') => {
                            // Mark is already set before this state is triggered.
                            self.consume();
                            self.state = TokenizationState::ScriptDataEscapeStart;
                        },
                        _ => {
                            self.state = TokenizationState::ScriptData;
                            
                        }
                    }
                }
                TokenizationState::ScriptDataEndTagOpen => unimplemented!("ScriptDataEndTagOpen"),
                TokenizationState::ScriptDataEndTagName => unimplemented!("ScriptDataEndTagName"),

                // https://html.spec.whatwg.org/#script-data-escape-start-state
                TokenizationState::ScriptDataEscapeStart => {
                    match self.peek() {
                        Some('-') => {
                            self.consume();
                            self.state = TokenizationState::ScriptDataEscapeStartDash;
                        },
                        _ => {
                            self.state = TokenizationState::ScriptData;
                        }

                    }
                },

                // https://html.spec.whatwg.org/#script-data-escape-start-dash-state
                TokenizationState::ScriptDataEscapeStartDash => {
                    match self.peek() {
                        Some('-') => {
                            self.consume();
                            self.state = TokenizationState::ScriptDataEscapedDashDash;
                        },
                        _ => {
                            self.state = TokenizationState::ScriptData;
                        }
                    }

                }
                TokenizationState::ScriptDataEscaped => unimplemented!("ScriptDataEscaped"),
                TokenizationState::ScriptDataEscapedDash => unimplemented!("ScriptDataEscapedDash"),

                TokenizationState::ScriptDataEscapedDashDash => {
                    match self.peek() {
                        Some('-') => {
                            self.consume();
                        },
                        Some('<') => {
                            self.consume();
                            self.state = TokenizationState::ScriptDataEscapedLessThanSign;
                        },
                        Some('>') => {
                            self.consume();
                            self.state = TokenizationState::ScriptData;
                        },
                        Some('\0') => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            self.state = TokenizationState::ScriptDataEscaped;
                        },
                        None => {
                            self.errors.push(Error::EofInScriptHtmlCommentLikeText);
                        },
                        Some(_) => {
                            self.consume();
                            self.state = TokenizationState::ScriptDataEscaped;
                        }
                    }
                },

                TokenizationState::ScriptDataEscapedLessThanSign => {
                    unimplemented!("ScriptDataEscapedLessThanSign")
                }
                TokenizationState::ScriptDataEscapedEndTagOpen => {
                    unimplemented!("ScriptDataEscapedEndTagOpen")
                }
                TokenizationState::ScriptDataEscapedEndTagName => {
                    unimplemented!("ScriptDataEscapedEndTagName")
                }
                TokenizationState::ScriptDataDoubleEscapeStart => {
                    unimplemented!("ScriptDataDoubleEscapeStart")
                }
                TokenizationState::ScriptDataDoubleEscaped => {
                    unimplemented!("ScriptDataDoubleEscaped")
                }
                TokenizationState::ScriptDataDoubleEscapedDash => {
                    unimplemented!("ScriptDataDoubleEscapedDash")
                }
                TokenizationState::ScriptDataDoubleEscapedDashDash => {
                    unimplemented!("ScriptDataDoubleEscapedDashDash")
                }
                TokenizationState::ScriptDataDoubleEscapedLessThanSign => {
                    unimplemented!("ScriptDataDoubleEscapedLessThanSign")
                }
                TokenizationState::ScriptDataDoubleEscapeEnd => {
                    unimplemented!("ScriptDataDoubleEscapeEnd")
                }

                // https://html.spec.whatwg.org/#before-attribute-name-state
                TokenizationState::BeforeAttributeName => {
                    match self.peek() {
                        Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                            self.consume();
                        }
                        Some('/') | Some('>') | None => {
                            self.state = TokenizationState::AfterAttributeName;
                        }
                        Some('=') => {
                            // Don't consume here, as the = needs to form part of the
                            // name.
                            self.errors
                                .push(Error::UnexpectedEqualsSignBeforeAttributeName);
                            self.mark = self.pos;
                            self.state = TokenizationState::AttributeName;
                        }
                        _ => {
                            self.mark = self.pos;
                            self.state = TokenizationState::AttributeName;
                        }
                    }
                }

                // https://html.spec.whatwg.org/#attribute-name-state
                TokenizationState::AttributeName => {
                    loop {
                        match self.peek() {
                            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') | Some('/')
                            | Some('>') | None => {
                                let name_slice = &self.input[self.mark..self.pos];
                                let final_name = if name_slice.contains('\0') {
                                    Cow::Owned(
                                        name_slice.replace('\0', "\u{FFFD}").to_ascii_lowercase(),
                                    )
                                } else {
                                    Cow::Owned(name_slice.to_ascii_lowercase())
                                };

                                // Check for duplicate attributes.
                                let tag = self.current_tag_buffer.as_mut().unwrap();
                                let name_already_exist = tag
                                    .attributes
                                    .iter()
                                    .any(|attr| attr.name.as_ref() == Some(&final_name));

                                if name_already_exist {
                                    self.errors.push(Error::DuplicateAttribute);
                                } else {
                                    tag.attributes.push(Attribute {
                                        name: Some(final_name),
                                        value: None,
                                    });
                                }
                                self.state = TokenizationState::AfterAttributeName;
                                break;
                            }
                            Some('=') => {
                                let name_slice = &self.input[self.mark..self.pos];
                                let final_name = if name_slice.contains('\0') {
                                    Cow::Owned(
                                        name_slice.replace('\0', "\u{FFFD}").to_ascii_lowercase(),
                                    )
                                } else {
                                    Cow::Owned(name_slice.to_ascii_lowercase())
                                };

                                // Check for duplicate attributes.
                                let tag = self.current_tag_buffer.as_mut().unwrap();
                                let name_already_exist = tag
                                    .attributes
                                    .iter()
                                    .any(|attr| attr.name.as_ref() == Some(&final_name));

                                if name_already_exist {
                                    self.errors.push(Error::DuplicateAttribute);
                                } else {
                                    tag.attributes.push(Attribute {
                                        name: Some(final_name),
                                        value: None,
                                    });
                                }

                                self.consume();
                                self.state = TokenizationState::BeforeAttributeValue;
                                break;
                            }
                            Some('\0') => {
                                self.consume();
                                self.errors.push(Error::UnexpectedNullCharacter);
                            }
                            Some('"') | Some('\'') | Some('<') => {
                                self.errors.push(Error::UnexpectedCharacterInAttributeName);
                            }
                            Some(_) => {
                                self.consume();
                            }
                        }
                    }
                }

                // https://html.spec.whatwg.org/#after-attribute-name-state
                TokenizationState::AfterAttributeName => match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                    }
                    Some('/') => {
                        self.consume();
                        self.state = TokenizationState::SelfClosingStartTag;
                    }
                    Some('=') => {
                        self.consume();
                        self.state = TokenizationState::BeforeAttributeValue;
                    }
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::StartTag(self.current_tag_buffer.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInTag);
                        return Some(HtmlToken::EndOfFile);
                    }
                    _ => {
                        let tag = self.current_tag_buffer.as_mut().unwrap();
                        tag.attributes.push(Attribute {
                            name: None,
                            value: None,
                        });
                        self.mark = self.pos;
                        self.state = TokenizationState::AttributeName;
                    }
                },

                // https://html.spec.whatwg.org/#before-attribute-state
                TokenizationState::BeforeAttributeValue => match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                    }
                    Some('"') => {
                        self.consume();
                        self.mark = self.pos;
                        self.state = TokenizationState::AttributeValueDoubleQuoted;
                    }
                    Some('\'') => {
                        self.consume();
                        self.mark = self.pos;
                        self.state = TokenizationState::AttributeValueSingleQuoted;
                    }
                    Some('>') => {
                        self.consume();
                        self.errors.push(Error::MissingAttributeValue);
                        self.state = TokenizationState::Data;

                        assert_ne!(
                            self.current_tag_buffer, None,
                            "Attempting to emit non initialised tag"
                        );

                        if self.is_current_tag_end {
                            self.is_current_tag_end = false;
                            return Some(HtmlToken::EndTag(
                                self.current_tag_buffer.take().unwrap(),
                            ));
                        } else {
                            return Some(HtmlToken::StartTag(
                                self.current_tag_buffer.take().unwrap(),
                            ));
                        }
                    }
                    _ => {
                        self.mark = self.pos;
                        self.state = TokenizationState::AttributeValueUnquoted;
                    }
                },

                // https://html.spec.whatwg.org/#attribute-value-double-quoted-state
                TokenizationState::AttributeValueDoubleQuoted => {
                    loop {
                        match self.peek() {
                            Some('"') => {
                                assert_ne!(
                                    self.current_tag_buffer, None,
                                    "Attempting to update attribute value of uninitialised tag."
                                );

                                let value_slice = &self.input[self.mark..self.pos];

                                if let Some(tag) = self.current_tag_buffer.as_mut() {
                                    // If value is not none, it means that the attribute
                                    // name was most probably a duplicate.
                                    if let Some(attribute) = tag.attributes.last_mut() && attribute.value.is_none() {
                                        attribute.value = Some(Cow::Borrowed(value_slice));
                                    }
                                }

                                self.consume();
                                self.state = TokenizationState::AfterAttributeValueQuoted;
                                break;
                            }
                            Some('&') => {
                                self.consume();
                                self.return_state = TokenizationState::AttributeValueSingleQuoted;
                                self.state = TokenizationState::CharacterReference;
                            }
                            Some('\0') => {
                                self.errors.push(Error::UnexpectedNullCharacter);
                                self.consume();
                            }
                            None => {
                                self.errors.push(Error::EofInTag);
                                return Some(HtmlToken::EndOfFile);
                            }
                            Some(_) => {
                                self.consume();
                            }
                        }
                    }
                },

                // https://html.spec.whatwg.org/#attribute-value-single-quoted-state
                TokenizationState::AttributeValueSingleQuoted => {
                    loop {
                        match self.peek() {
                            Some('\'') => {
                                assert_ne!(
                                    self.current_tag_buffer, None,
                                    "Attempting to update attribute value of uninitialised tag."
                                );

                                let value_slice = &self.input[self.mark..self.pos];

                                if let Some(tag) = self.current_tag_buffer.as_mut() {
                                    // If value is not none, it means that the attribute
                                    // name was most probably a duplicate.
                                    if let Some(attribute) = tag.attributes.last_mut() && attribute.value.is_none() {
                                        attribute.value = Some(Cow::Borrowed(value_slice));
                                    }
                                }

                                self.consume();
                                self.state = TokenizationState::AfterAttributeValueQuoted;
                                break;
                            }
                            Some('&') => {
                                self.consume();
                                self.return_state = TokenizationState::AttributeValueSingleQuoted;
                                self.state = TokenizationState::CharacterReference;
                            }
                            Some('\0') => {
                                self.errors.push(Error::UnexpectedNullCharacter);
                                self.consume();
                            }
                            None => {
                                self.errors.push(Error::EofInTag);
                                return Some(HtmlToken::EndOfFile);
                            }
                            Some(_) => {
                                self.consume();
                            }
                        }
                    }
                }

                TokenizationState::AttributeValueUnquoted => match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        assert_ne!(
                            self.current_tag_buffer, None,
                            "Attempting to update attribute value of uninitialised tag."
                        );

                        let value_slice = &self.input[self.mark..self.pos];

                        if let Some(tag) = self.current_tag_buffer.as_mut() && let Some(attribute) = tag.attributes.last_mut(){
                            attribute.value = Some(Cow::Borrowed(value_slice));
                        }
                        self.consume();
                        self.state = TokenizationState::BeforeAttributeName;
                    }
                    Some('&') => {
                        self.return_state = TokenizationState::AttributeValueUnquoted;
                        self.state = TokenizationState::CharacterReference;
                    }
                    Some('>') => {
                        let value_slice = &self.input[self.mark..self.pos];

                        if let Some(tag) = self.current_tag_buffer.as_mut() && let Some(attribute) = tag.attributes.last_mut(){
                            attribute.value = Some(Cow::Borrowed(value_slice));
                        }
                        self.consume();
                        self.state = TokenizationState::Data;

                        if self.is_current_tag_end {
                            self.is_current_tag_end = false;
                            return Some(HtmlToken::EndTag(
                                self.current_tag_buffer.take().unwrap(),
                            ));
                        } else {
                            return Some(HtmlToken::StartTag(
                                self.current_tag_buffer.take().unwrap(),
                            ));
                        }
                    }
                    Some('\0') => {
                        self.consume();
                        self.errors.push(Error::UnexpectedNullCharacter);
                    }
                    Some('"') | Some('\'') | Some('<') | Some('=') | Some('`') => {
                        self.consume();
                        self.errors
                            .push(Error::UnexpectedCharacterInUnquotedAttributeValue);
                    }
                    None => {
                        return Some(HtmlToken::EndOfFile);
                    }
                    Some(_) => {
                        self.consume();
                    }
                },

                // https://html.spec.whatwg.org/#after-attribute-value-quoted-state
                TokenizationState::AfterAttributeValueQuoted => match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.state = TokenizationState::BeforeAttributeName;
                    }
                    Some('/') => {
                        self.consume();
                        self.state = TokenizationState::SelfClosingStartTag;
                    }
                    Some('>') => {
                        self.consume();
                        self.state = TokenizationState::Data;

                        if self.is_current_tag_end {
                            self.is_current_tag_end = false;
                            return Some(HtmlToken::EndTag(
                                self.current_tag_buffer.take().unwrap(),
                            ));
                        } else {
                            return Some(HtmlToken::StartTag(
                                self.current_tag_buffer.take().unwrap(),
                            ));
                        }
                    }
                    None => {
                        self.errors.push(Error::EofInTag);
                        return Some(HtmlToken::EndOfFile);
                    }
                    _ => {
                        self.errors.push(Error::MissingWhitespaceBetweenAttributes);
                        self.state = TokenizationState::BeforeAttributeName;
                    }
                },

                // https://html.spec.whatwg.org/#self-closing-start-tag-state
                TokenizationState::SelfClosingStartTag => match self.peek() {
                    Some('>') => {
                        self.consume();
                        if let Some(tag) = self.current_tag_buffer.as_mut() {
                            tag.self_closing_tag = Some(true);
                        }
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::StartTag(self.current_tag_buffer.take().unwrap()));
                    }
                    None => {
                        self.errors.push(Error::EofInTag);
                        return Some(HtmlToken::EndOfFile);
                    }
                    _ => {
                        self.errors.push(Error::UnexpectedSolidusInTag);
                        self.state = TokenizationState::BeforeAttributeName;
                    }
                },

                // https://html.spec.whatwg.org/#bogus-comment-state
                TokenizationState::BogusComment => loop {
                    match self.peek() {
                        Some('>') => {
                            self.state = TokenizationState::Data;
                            let comment_slice = &self.input[self.mark..self.pos];
                            self.consume();
                            return Some(HtmlToken::Comment(Cow::Owned(
                                comment_slice.replace('\0', "\u{FFFD}"),
                            )));
                        }
                        None => {
                            let comment_slice = &self.input[self.mark..self.pos];
                            self.consume();
                            self.state = TokenizationState::Data;
                            return Some(HtmlToken::Comment(Cow::Borrowed(comment_slice)));
                        }
                        Some('\0') => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            self.consume();
                        }
                        Some(_) => {
                            self.consume();
                        }
                    }
                },

                // https://html.spec.whatwg.org/#markup-declaration-open-state
                TokenizationState::MarkupDeclarationOpen => {
                    let remaining = &self.input[self.pos..];

                    if remaining.starts_with("--") {
                        self.consume();
                        self.consume();
                        self.mark = self.pos;
                        self.state = TokenizationState::CommentStart;
                    } else if remaining.len() > 7 && remaining[..7].eq_ignore_ascii_case("doctype")
                    {
                        for _ in 0..7 {
                            self.consume();
                        }
                        self.state = TokenizationState::Doctype;
                    } else if remaining.starts_with("[CDATA[") {
                        todo!()
                    } else {
                        self.errors.push(Error::IncorrectlyOpenedComment);
                        self.mark = self.pos;
                        self.state = TokenizationState::BogusComment;
                    }
                }

                TokenizationState::CommentStart => match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::CommentStartDash;
                    }
                    Some('>') => {
                        self.errors.push(Error::AbruptClosingOfEmptyComment);
                        let comment_slice = &self.input[self.mark..self.pos];
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Comment(Cow::Owned(
                            comment_slice.replace('\0', "\u{FFFD}"),
                        )));
                    }
                    _ => {
                        self.state = TokenizationState::Comment;
                    }
                },

                TokenizationState::CommentStartDash => match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::CommentEnd;
                    }
                    Some('>') => {
                        self.errors.push(Error::AbruptClosingOfEmptyComment);
                        let comment_slice = &self.input[self.mark..self.pos - 1];
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Comment(Cow::Owned(
                            comment_slice.replace('\0', "\u{FFFD}"),
                        )));
                    }
                    None => {
                        self.errors.push(Error::EofInComment);
                        let comment_slice = &self.input[self.mark..self.pos];
                        self.consume();
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Comment(Cow::Owned(
                            comment_slice.replace('\0', "\u{FFFD}"),
                        )));
                    }
                    Some(_) => {
                        self.state = TokenizationState::Comment;
                    }
                },

                TokenizationState::Comment => {
                    loop {
                        match self.peek() {
                            Some('<') => {
                                self.consume();
                                self.state = TokenizationState::CommentLessThanSign;
                                break;
                            }
                            Some('-') => {
                                self.consume();
                                self.state = TokenizationState::CommentEndDash;
                                break;
                            }
                            Some('\0') => {
                                self.consume();
                                self.errors.push(Error::UnexpectedNullCharacter);
                            }
                            None => {
                                //self.errors.push(Error::EofInTag);
                                let comment_slice = &self.input[self.mark..self.pos];
                                self.consume();
                                self.state = TokenizationState::Data;
                                return Some(HtmlToken::Comment(Cow::Owned(
                                    comment_slice.replace('\0', "\u{FFFD}"),
                                )));
                            }
                            _ => {
                                self.consume();
                            }
                        }
                    }
                }

                TokenizationState::CommentLessThanSign => match self.peek() {
                    Some('!') => {
                        self.consume();
                        self.state = TokenizationState::CommentLessThanSignBang;
                    }
                    Some('<') => {
                        self.consume();
                    }
                    _ => {
                        self.state = TokenizationState::Comment;
                    }
                },

                TokenizationState::CommentLessThanSignBang => match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::CommentLessThanSignBangDash;
                    }
                    _ => {
                        self.state = TokenizationState::Comment;
                    }
                },

                TokenizationState::CommentLessThanSignBangDash => match self.peek() {
                    Some('-') => {
                        self.consume();
                        self.state = TokenizationState::CommentLessThanSignBangDashDash;
                    }
                    _ => {
                        self.state = TokenizationState::Comment;
                    }
                },

                TokenizationState::CommentLessThanSignBangDashDash => match self.peek() {
                    Some('>') | None => {
                        self.state = TokenizationState::CommentEnd;
                    }
                    _ => {
                        self.errors.push(Error::NestedComment);
                        self.state = TokenizationState::CommentEnd;
                    }
                },

                TokenizationState::CommentEndDash => {
                    match self.peek() {
                        Some('-') => {
                            self.consume();
                            self.state = TokenizationState::CommentEnd;
                        }
                        None => {
                            self.consume();
                            self.errors.push(Error::EofInComment);
                            // Use pos-1, because in order to get to this state
                            // we had to consume a '-'.
                            let comment_slice = &self.input[self.mark..self.pos - 1];
                            return Some(HtmlToken::Comment(Cow::Owned(
                                comment_slice.replace('\0', "\u{FFFD}"),
                            )));
                        }
                        _ => {
                            self.state = TokenizationState::CommentStart;
                        }
                    }
                }

                TokenizationState::CommentEnd => {
                    match self.peek() {
                        Some('>') => {
                            // Use pos-2, because in order to get to this state
                            // we had to consume a '--'.
                            let comment_slice = &self.input[self.mark..self.pos - 2];
                            self.consume();
                            self.state = TokenizationState::Data;
                            return Some(HtmlToken::Comment(Cow::Owned(
                                comment_slice.replace('\0', "\u{FFFD}"),
                            )));
                        }
                        Some('!') => {
                            self.consume();
                            self.state = TokenizationState::CommentEndBang;
                        }
                        Some('-') => {
                            self.consume();
                        }
                        None => {
                            self.errors.push(Error::EofInComment);
                            let comment_slice = &self.input[self.mark..self.pos - 2];
                            self.consume();
                            self.state = TokenizationState::Data;
                            return Some(HtmlToken::Comment(Cow::Owned(
                                comment_slice.replace('\0', "\u{FFFD}"),
                            )));
                        }
                        _ => {
                            self.consume();
                            self.state = TokenizationState::Comment;
                        }
                    }
                }

                TokenizationState::CommentEndBang => unimplemented!("CommentEndBang"),

                // https://html.spec.whatwg.org/#doctype-state
                TokenizationState::Doctype => match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.state = TokenizationState::BeforeDoctypeName;
                    }
                    Some('>') => {
                        self.state = TokenizationState::BeforeDoctypeName;
                    }
                    None => {
                        self.errors.push(Error::EofInDoctype);
                        self.consume();
                        return Some(HtmlToken::Doctype(Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks_flag: true,
                        }));
                    }
                    Some(_) => {
                        self.errors.push(Error::MissingWhitespaceBeforeDoctypeName);
                        self.state = TokenizationState::BeforeDoctypeName;
                    }
                },

                // https://html.spec.whatwg.org/#before-doctype-name-state
                TokenizationState::BeforeDoctypeName => match self.peek() {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                    }
                    Some('\0') => {
                        self.errors.push(Error::UnexpectedNullCharacter);
                        self.current_doctype_buffer = Some(Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks_flag: false,
                        });
                        self.mark = self.pos;
                        self.state = TokenizationState::DoctypeName;
                    }
                    Some('>') => {
                        self.consume();
                        self.errors.push(Error::MissingDoctypeName);
                        self.state = TokenizationState::Data;
                        return Some(HtmlToken::Doctype(Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks_flag: true,
                        }));
                    }
                    None => {
                        self.errors.push(Error::EofInTag);
                        return Some(HtmlToken::Doctype(Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks_flag: true,
                        }));
                    }
                    Some(_) => {
                        self.current_doctype_buffer = Some(Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks_flag: false,
                        });
                        self.mark = self.pos;
                        self.state = TokenizationState::DoctypeName;
                    }
                },

                // https://html.spec.whatwg.org/#doctype-name-state
                TokenizationState::DoctypeName => loop {
                    match self.peek() {
                        Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                            self.consume();
                            self.state = TokenizationState::AfterDoctypeName;
                            break;
                        }
                        Some('>') => {
                            self.state = TokenizationState::Data;
                            let name_slice = &self.input[self.mark..self.pos];
                            self.consume();
                            if let Some(doctype_buffer) = self.current_doctype_buffer.as_mut() && doctype_buffer.name.is_none() {
                                doctype_buffer.name = Some(Cow::Owned(
                                    name_slice.replace('\0', "\u{FFFD}").to_ascii_lowercase(),
                                ));
                                let doctype = Some(HtmlToken::Doctype(
                                    self.current_doctype_buffer.take().unwrap(),
                                ));
                                self.current_doctype_buffer = None;
                                return doctype;
                            }
                        }
                        Some('\0') => {
                            self.errors.push(Error::UnexpectedNullCharacter);
                            self.consume();
                        }
                        None => {
                            self.errors.push(Error::EofInDoctype);
                            let name_slice = &self.input[self.mark..self.pos];
                            self.consume();
                            if let Some(doctype_buffer) = self.current_doctype_buffer.as_mut() && doctype_buffer.name.is_none(){
                                doctype_buffer.name = Some(Cow::Owned(
                                    name_slice.replace('\0', "\u{FFFD}").to_ascii_lowercase(),
                                ));
                                doctype_buffer.force_quirks_flag = true;
                                let doctype = Some(HtmlToken::Doctype(
                                    self.current_doctype_buffer.take().unwrap(),
                                ));
                                self.current_doctype_buffer = None;
                                return doctype;
                            }
                            return Some(HtmlToken::EndOfFile);
                        }
                        Some(_) => {
                            self.consume();
                        }
                    }
                },

                TokenizationState::AfterDoctypeName => unimplemented!("AfterDoctypeName"),
                TokenizationState::AfterDoctypePublicKeyword => {
                    unimplemented!("AfterDoctypePublicKeyword")
                }
                TokenizationState::BeforeDoctypePublicIdentifier => {
                    unimplemented!("BeforeDoctypePublicIdentifier")
                }
                TokenizationState::DoctypePublicIdentifierDoubleQuoted => {
                    unimplemented!("DoctypePublicIdentifierDoubleQuoted")
                }
                TokenizationState::DoctypePublicIdentifierSingleQuoted => {
                    unimplemented!("DoctypePublicIdentifierSingleQuoted")
                }
                TokenizationState::AfterDoctypePublicIdentifier => {
                    unimplemented!("AfterDoctypePublicIdentifier")
                }
                TokenizationState::BetweenDoctypePublicAndSystemIdentifiers => {
                    unimplemented!("BetweenDoctypePublicAndSystemIdentifiers")
                }
                TokenizationState::AfterDoctypeSystemKeyword => {
                    unimplemented!("AfterDoctypeSystemKeyword")
                }
                TokenizationState::BeforeDoctypeSystemIdentifier => {
                    unimplemented!("BeforeDoctypeSystemIdentifier")
                }
                TokenizationState::DoctypeSystemIdentifierDoubleQuoted => {
                    unimplemented!("DoctypeSystemIdentifierDoubleQuoted")
                }
                TokenizationState::DoctypeSystemIdentifierSingleQuoted => {
                    unimplemented!("DoctypeSystemIdentifierSingleQuoted")
                }
                TokenizationState::AfterDoctypeSystemIdentifier => {
                    unimplemented!("AfterDoctypeSystemIdentifier")
                }
                TokenizationState::BogusDoctype => unimplemented!("BogusDoctype"),
                TokenizationState::CdataSection => unimplemented!("CdataSection"),
                TokenizationState::CdataSectionBracket => unimplemented!("CdataSectionBracket"),
                TokenizationState::CdataSectionEnd => unimplemented!("CdataSectionEnd"),
                TokenizationState::ProcessingInstructionOpen => {
                    unimplemented!("ProcessingInstructionOpen")
                }
                TokenizationState::ProcessingInstructionTarget => {
                    unimplemented!("ProcessingInstructionTarget")
                }
                TokenizationState::AfterProcessingInstructionTarget => {
                    unimplemented!("AfterProcessingInstructionTarget")
                }
                TokenizationState::ProcessingInstructionData => {
                    unimplemented!("ProcessingInstructionData")
                }
                TokenizationState::ProcessingInstructionQuestionable => {
                    unimplemented!("ProcessingInstructionQuestionable")
                }

                // https://html.spec.whatwg.org/#character-reference-state
                TokenizationState::CharacterReference => {
                    // mark is set to pos -1, as the ampersand
                    // was consumed but need to be considered
                    // when flushing. Or does it?
                    self.mark = self.pos - 1;
                    match self.peek() {
                        Some(c) if Self::is_ascii_alpha(c) => {
                            self.state = TokenizationState::NamedCharacterReference;
                        }
                        Some('#') => {
                            self.consume();
                            self.state = TokenizationState::NumericCharacterReference;
                        }
                        _ => {
                            self.state = self.return_state;
                            return Some(HtmlToken::Character(Cow::Borrowed(
                                &self.input[self.mark..self.pos],
                            )));
                        }
                    }
                }

                // https://html.spec.whatwg.org/#named-character-reference-state
                TokenizationState::NamedCharacterReference => loop {
                    match self.peek() {
                        Some(';') => {
                            self.consume();

                            let character_reference_slice = &self.input[self.mark..self.pos];
                            let matched_reference = match character_reference_slice {
                                "&lt;" => "<",
                                _ => character_reference_slice,
                            };
                            self.state = self.return_state;
                            return Some(HtmlToken::Character(Cow::Borrowed(&matched_reference)));
                        }
                        _ => {
                            self.consume();
                        }
                    }
                },

                // https://html.spec.whatwg.org/#ambiguous-ampersand-state
                TokenizationState::AmbiguousAmpersand => {
                    unimplemented!("AmbiguousAmpersand")
                }

                TokenizationState::NumericCharacterReference => {
                    unimplemented!("NumericCharacterReference")
                }

                TokenizationState::HexadecimalCharacterReferenceStart => {
                    unimplemented!("HexadecimalCharacterReferenceStart")
                }

                TokenizationState::HexadecimalCharacterReference => {
                    unimplemented!("HexadecimalCharacterReference")
                }

                TokenizationState::DecimalCharacterReference => {
                    unimplemented!("DecimalCharacterReference")
                }

                TokenizationState::NumericCharacterReferenceEnd => {
                    unimplemented!("NumericCharacterReferenceEnd")
                }
            }
        }
    }
}
