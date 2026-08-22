/*
use crate::html::{errors::{Error, TokenErrorKind}, tokens::HtmlToken};


use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum HtmlNode<'a> {
    Element {
        name: &'a str,
        attributes: HashMap<&'a str, Option<&'a str>>, // None for BoolAttributes
        children: Vec<HtmlNode<'a>>,
    },
    Text(&'a str),
    Comment(&'a str),
    Declaration(&'a str),
}

pub struct HtmlParser<'a, I: Iterator<Item = Result<HtmlToken<'a>, Error>>> {
    tokens: I,
    peeked: Option<Result<HtmlToken<'a>, Error>>,
}

impl<'a, I: Iterator<Item = Result<HtmlToken<'a>, Error>>> HtmlParser<'a, I> {
    pub fn new(mut tokens: I) -> Self {
        let peeked = tokens.next();
        Self { tokens, peeked }
    }

    fn next_token(&mut self) -> Option<Result<HtmlToken<'a>, Error>> {
        self.peeked.take().or_else(|| self.tokens.next())
    }

    /// Parses the entire token stream into a list of root-level AST nodes.
    pub fn parse(&mut self) -> Result<Vec<HtmlNode<'a>>, Error> {
        let mut nodes = Vec::new();

        while let Some(token_result) = self.next_token() {
            let token = token_result?;
            if let Some(node) = self.parse_node(token)? {
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    fn parse_node(&mut self, token: HtmlToken<'a>) -> Result<Option<HtmlNode<'a>>, Error> {
        match token {
            HtmlToken::Declaration(dec) => Ok(Some(HtmlNode::Declaration(dec))),
            HtmlToken::Comment(com) => Ok(Some(HtmlNode::Comment(com))),
            HtmlToken::Text(txt) => Ok(Some(HtmlNode::Text(txt))),
            HtmlToken::StartTag(name) => {
                let mut attributes = HashMap::new();
                let mut self_closing = false;

                // Consume attributes and the tag closing token
                loop {
                    match self.next_token() {
                        Some(Ok(HtmlToken::Attribute { name: attr_name, value })) => {
                            attributes.insert(attr_name, Some(value));
                        }
                        Some(Ok(HtmlToken::BoolAttribute(attr_name))) => {
                            attributes.insert(attr_name, None);
                        }
                        Some(Ok(HtmlToken::TagEnd { self_closing: sc })) => {
                            self_closing = sc;
                            break;
                        }
                        Some(Err(e)) => return Err(e),
                        _ => {
                            return Err(Error::UnterminatedToken {
                                pos: 0,
                                kind: TokenErrorKind::Tag,
                            })
                        }
                    }
                }

                // If it's a self-closing tag (e.g., <br />), it has no children
                if self_closing {
                    return Ok(Some(HtmlNode::Element {
                        name,
                        attributes,
                        children: Vec::new(),
                    }));
                }

                // Otherwise, parse children until we hit the matching EndTag
                let mut children = Vec::new();
                loop {
                    let next = match self.next_token() {
                        Some(t) => t?,
                        None => break,
                    };

                    if let HtmlToken::EndTag(end_name) = next {
                        if end_name != name {
                            // Mismatched tag error handling can go here
                        }
                        break;
                    }

                    if let Some(child) = self.parse_node(next)? {
                        children.push(child);
                    }
                }

                Ok(Some(HtmlNode::Element {
                    name,
                    attributes,
                    children,
                }))
            }
            // End tags or stray tag ends at the root level can be ignored or handled
            _ => Ok(None),
        }
    }
}

use std::fmt::Write;

impl<'a> HtmlNode<'a> {
    /// Recursively prints a visual tree representation of the AST.
    pub fn display_tree(&self) -> String {
        let mut output = String::new();
        self.format_tree(&mut output, "", true);
        output
    }

    fn format_tree(&self, f: &mut String, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };

        match self {
            HtmlNode::Element { name, attributes, children } => {
                // Format attributes nicely
                let mut attrs_str = String::new();
                for (k, v) in attributes {
                    if let Some(val) = v {
                        let _ = write!(attrs_str, " {}=\"{}\"", k, val);
                    } else {
                        let _ = write!(attrs_str, " {}", k); // Bool attribute
                    }
                }
                let _ = writeln!(f, "{}{}<{}{}>", prefix, connector, name, attrs_str);

                // Setup prefix for children
                let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                let child_count = children.len();
                for (i, child) in children.iter().enumerate() {
                    child.format_tree(f, &new_prefix, i == child_count - 1);
                }
            }
            HtmlNode::Text(text) => {
                let clean_text = text.replace('\n', "");
                let _ = writeln!(f, "{}{}[Text: \"{}\"]", prefix, connector, clean_text.trim());
            }
            HtmlNode::Comment(comment) => {
                let clean_comment = comment.replace('\n', "");
                let _ = writeln!(f, "{}{}[Comment: <!-- {} -->]", prefix, connector, clean_comment);
            }
            HtmlNode::Declaration(dec) => {
                let clean_dec = dec.replace('\n', "");
                let _ = writeln!(f, "{}{}[Declaration: <?{}?>]", prefix, connector, clean_dec);
            }
        }
    }
}*/
