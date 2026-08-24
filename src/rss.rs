// https://www.rssboard.org/rss-specification

use std::borrow::Cow;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    XmlToken(crate::xml::errors::Error),
    UnterminatedEscapeChar(String),
    UnknownEscapeChar(String),
    Utf8Error(std::string::FromUtf8Error),
    RequestError(ureq::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::XmlToken(err) => write!(f, "{}", err),
            Self::UnterminatedEscapeChar(err) => {
                write!(f, "Unterminated escape character: {}", err)
            }
            Self::UnknownEscapeChar(err) => write!(f, "Unknown escape char: {}", err),
            Self::Utf8Error(err) => write!(f, "{}", err),
            Self::RequestError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::xml::errors::Error> for Error {
    fn from(value: crate::xml::errors::Error) -> Self {
        Self::XmlToken(value)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
        Self::RequestError(value)
    }
}

enum RssElement {
    Rss,
    Channel,
    Item,
}

enum RssTag {
    Title,
    Link,
    Description,
    Author,
    Language,
}

#[derive(Debug, Default)]
pub struct RssItem<'a> {
    title: Option<Cow<'a, str>>,
    link: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
    author: Option<Cow<'a, str>>,
}

#[derive(Debug, Default)]
pub struct RssChannel<'a> {
    pub title: Option<Cow<'a, str>>,
    pub link: Option<Cow<'a, str>>,
    pub description: Option<Cow<'a, str>>,
    pub language: Option<Cow<'a, str>>,
}

#[derive(Debug)]
pub struct RssFeed<'a> {
    pub version: Cow<'a, str>,
    pub channel: RssChannel<'a>,
    pub items: Vec<RssItem<'a>>,
    //language: Option<&'a str>,
    // TODO: Incorporate other optional fieldsKets .
}

impl<'a> Display for RssItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = self.title.as_deref().unwrap_or("Untitled");
        write!(f, "Title: {}", title)?;

        if let Some(link) = self.link.as_deref() {
            write!(f, "\nLink: {}", link)?;
        };

        if let Some(author) = self.author.as_deref() {
            write!(f, "\nAuthor: {}", author)?;
        }

        if let Some(description) = self.description.as_deref() {
            write!(f, "\nDescription: {}", description)?;
        }

        writeln!(f)?;

        Ok(())
    }
}

// Will fail on unterminated escape char.
fn unescape_xml_control_char<'a>(input: &'a str) -> Result<Cow<'a, str>, Error> {
    if !input.contains('&') {
        return Ok(Cow::Borrowed(input));
    }

    let bytes = input.as_bytes();
    let mut result = Vec::with_capacity(input.len());

    let mut i = 0;
    while i < input.len() {
        if bytes[i] == b'&' {
            // Find the control char end index.
            let current_idx = i;
            while i < input.len() && bytes[i] != b';' {
                i += 1;
            }

            if i >= input.len() {
                return Err(Error::UnterminatedEscapeChar(
                    String::from_utf8_lossy(&bytes[current_idx..]).into(),
                ));
            }

            let unescape_char = match &bytes[current_idx..=i] {
                b"&lt;" => b'<',
                b"&gt;" => b'>',
                b"&quot;" => b'"',
                b"&apos;" => b'\'',
                b"&amp;" => b'&',
                _ => {
                    return Err(Error::UnknownEscapeChar(
                        String::from_utf8_lossy(&bytes[current_idx..=i]).into(),
                    ));
                }
            };
            result.push(unescape_char);
        } else {
            result.push(bytes[i]);
        }
        i += 1;
    }
    let s = String::from_utf8(result)?;
    Ok(Cow::Owned(s))
}

impl<'a> RssItem<'a> {
    pub fn follow_link(&self) -> Result<String, Error> {
        if let Some(link) = self.link.as_deref() {
            let mut response = ureq::get(link).call()?;

            match response.status() {
                ureq::http::StatusCode::OK => {
                    let body = response.body_mut().read_to_string()?;
                    return Ok(body);
                }

                _ => unimplemented!("Update status code match"),
            }
        }

        todo!()
    }
}

impl<'a> RssFeed<'a> {
    pub fn from_tokenizer(
        tokenizer: &mut crate::xml::tokens::Tokenizer<'a>,
    ) -> Result<RssFeed<'a>, Error> {
        let mut feed_version = Option::<&'a str>::None;
        let mut channel = Option::<RssChannel>::None;
        let mut items = Vec::new();

        let mut current_element = Option::<RssElement>::None;
        let mut current_tag = Option::<RssTag>::None;
        let mut current_item = Option::<RssItem>::None;

        //while let Some(token_result) = tokenizer.next() {
        for token_result in tokenizer.by_ref() {
            let token = token_result?;

            match token {
                crate::xml::tokens::Token::StartTag("rss") => {
                    current_element = Some(RssElement::Rss);
                }

                crate::xml::tokens::Token::StartTag("channel") => {
                    channel = Some(RssChannel::default());
                    current_element = Some(RssElement::Channel);
                }

                crate::xml::tokens::Token::StartTag("item") => {
                    current_item = Some(RssItem::default());
                    current_element = Some(RssElement::Item);
                }

                crate::xml::tokens::Token::StartTag("title") => {
                    current_tag = Some(RssTag::Title);
                }

                crate::xml::tokens::Token::StartTag("link") => {
                    current_tag = Some(RssTag::Link);
                }

                crate::xml::tokens::Token::StartTag("description") => {
                    current_tag = Some(RssTag::Description);
                }

                crate::xml::tokens::Token::StartTag("author") => {
                    current_tag = Some(RssTag::Author);
                }

                crate::xml::tokens::Token::StartTag("language") => {
                    current_tag = Some(RssTag::Language);
                }

                crate::xml::tokens::Token::EndTag("rss") => {
                    current_element = None;
                }

                crate::xml::tokens::Token::EndTag("channel") => {
                    current_element = Some(RssElement::Rss);
                }

                crate::xml::tokens::Token::EndTag("item") => {
                    if let Some(item) = current_item.take() {
                        items.push(item);
                        current_element = Some(RssElement::Channel);
                    }
                }

                crate::xml::tokens::Token::EndTag(
                    "title" | "link" | "description" | "author" | "language",
                ) => {
                    current_tag = None;
                }

                crate::xml::tokens::Token::Attribute { name, value } => match current_element {
                    Some(RssElement::Rss) if name == "version" => feed_version = Some(value),
                    _ => continue,
                },

                crate::xml::tokens::Token::Text(text) => {
                    let cow_text = unescape_xml_control_char(text)?;

                    match (&current_element, &current_tag) {
                        (Some(RssElement::Channel), Some(RssTag::Title)) => {
                            if let Some(ref mut channel) = channel {
                                channel.title = Some(cow_text)
                            }
                        }

                        (Some(RssElement::Channel), Some(RssTag::Link)) => {
                            if let Some(ref mut channel) = channel {
                                channel.link = Some(cow_text)
                            }
                        }

                        (Some(RssElement::Channel), Some(RssTag::Description)) => {
                            if let Some(ref mut channel) = channel {
                                channel.description = Some(cow_text)
                            }
                        }

                        (Some(RssElement::Channel), Some(RssTag::Language)) => {
                            if let Some(ref mut channel) = channel {
                                channel.language = Some(cow_text)
                            }
                        }

                        (Some(RssElement::Item), Some(RssTag::Title)) => {
                            if let Some(ref mut item) = current_item {
                                item.title = Some(cow_text);
                            }
                        }
                        (Some(RssElement::Item), Some(RssTag::Link)) => {
                            if let Some(ref mut item) = current_item {
                                item.link = Some(cow_text);
                            }
                        }
                        (Some(RssElement::Item), Some(RssTag::Description)) => {
                            if let Some(ref mut item) = current_item {
                                item.description = Some(cow_text);
                            }
                        }
                        (Some(RssElement::Item), Some(RssTag::Author)) => {
                            if let Some(ref mut item) = current_item {
                                item.author = Some(cow_text);
                            }
                        }

                        _ => continue, //FIXME: Handle these explicitly rather than silently.
                    }
                }

                crate::xml::tokens::Token::CharacterData(text) => {
                    let cow_text = unescape_xml_control_char(text)?;

                    match (&current_element, &current_tag) {
                        (Some(RssElement::Channel), Some(RssTag::Title)) => {
                            if let Some(ref mut channel) = channel {
                                channel.title = Some(cow_text)
                            }
                        }

                        (Some(RssElement::Channel), Some(RssTag::Link)) => {
                            if let Some(ref mut channel) = channel {
                                channel.link = Some(cow_text)
                            }
                        }

                        (Some(RssElement::Channel), Some(RssTag::Description)) => {
                            if let Some(ref mut channel) = channel {
                                channel.description = Some(cow_text)
                            }
                        }

                        (Some(RssElement::Channel), Some(RssTag::Language)) => {
                            if let Some(ref mut channel) = channel {
                                channel.language = Some(cow_text)
                            }
                        }

                        (Some(RssElement::Item), Some(RssTag::Title)) => {
                            if let Some(ref mut item) = current_item {
                                item.title = Some(cow_text);
                            }
                        }
                        (Some(RssElement::Item), Some(RssTag::Link)) => {
                            if let Some(ref mut item) = current_item {
                                item.link = Some(cow_text);
                            }
                        }
                        (Some(RssElement::Item), Some(RssTag::Description)) => {
                            if let Some(ref mut item) = current_item {
                                item.description = Some(cow_text);
                            }
                        }
                        (Some(RssElement::Item), Some(RssTag::Author)) => {
                            if let Some(ref mut item) = current_item {
                                item.author = Some(cow_text);
                            }
                        }

                        _ => continue, //FIXME: Handle these explicitly rather than silently.
                    }
                }

                _ => continue, //FIXME: Handle these explicitly rather than silently.
            }
        }

        let unwrapped_version = match feed_version {
            Some(version) => Cow::Borrowed(version),
            None => panic!("No version found"),
        };

        let unwrapped_channel = match channel {
            Some(channel) => channel,
            None => panic!("No channel tag found"),
        };

        Ok(RssFeed {
            version: unwrapped_version,
            channel: unwrapped_channel,
            items,
        })
    }
}
