pub enum Error {
    UnexpectedNullCharacter

}

pub enum XmlToken<'a> {

    Declaration { 
        name: &'a str,
        version: &'a str,
        encoding: &'a str,
    },

    ProcessingInstruction {
        target: &'a str,
        data: &'a str,
    },

    DocumentType {
        name: &'a str,
        external_id: &'a str,
    },

    EntityDeclaration {
        name: &'a str,
        definition: &'a str,
    },
    
    // FIXME: Incorporate later.
    //AttrbuteListDeclaration,
    
    DocumentTypeEnd,

    StartTag {
        name: &'a str,
    },

    Attribute {
        name: &'a str,
        value: &'a str,
    },

    TagEnd {
        self_closing: bool,
    },

    EndTag {
        name: &'a str,
    },

    Text(&'a str ),

    Comment(&'a str),
}

pub enum XmlState {
    Normal,

    TagOpen,

    Declaration,
}

pub struct XmlTokenizer<'a> {
    input: &'a str,

    chars: std::str::CharIndices<'a>,

    current: Option<(usize, char)>,

    state: XmlState,

    mark: usize,

    depth: usize,
}

impl<'a> From<&'a str> for XmlTokenizer<'a> {
    fn from(value: &'a str) -> Self {
        let mut chars = value.char_indices();
        let current = chars.next();

        Self {
            input: value,
            chars,
            current,
            state: XmlState::Normal,
            mark: 0,
            depth: 0,
        }
    }
}
impl<'a> XmlTokenizer<'a> {

    #[inline]
    fn pos(&self) -> usize {
        self.current.map_or(self.input.len(), |(offset, _)| offset)
    }

    #[inline]
    fn mark(&mut self) {
        self.mark = self.pos();
    }

    #[inline]
    fn peek(&self) -> Option<char> {
        self.current.map(|(_, c)| c)
    }

    #[inline]
    fn consume(&mut self) -> Option<char> {
        let (_, c) = self.current?;
        self.current = self.chars.next();
        Some(c)
    }

    #[inline]
    fn slice_from_mark(&self) -> &'a str {
        let end = self.pos();
        &self.input[self.mark..end]
    }

    fn consume_char_run(&mut self, stop: impl Fn(char) -> bool) -> Option<&'a str> {
        self.mark();

        while let Some(c) = self.peek() {
            if stop(c) {
                break;
            }
            self.consume();
        }

        if self.pos() > self.mark {
            Some(self.slice_from_mark())
        } else {
            None
        }
    }
}

impl<'a> Iterator for XmlTokenizer<'a> {
    type Item = Result<XmlToken<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {

        if self.pos() >= self.input.len() {
            return None;
        }

        match self.state {
            XmlState::Normal => {
                if let Some(tok) = self.consume_char_run(|c| matches!(c, '<' | '\0')) {
                    return Some(Ok(XmlToken::Text(self.slice_from_mark())));
                }


                match self.consume() {
                    Some('<') => {
                        self.state = XmlState::TagOpen;
                    },
                    Some('\0') => {
                        return Some(Err(Error::UnexpectedNullCharacter));
                    }
                    None => {
                        return Some(Ok)
                    }

                }
            }




        }
    }
}
