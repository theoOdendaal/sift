use std::borrow::Cow;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    XmlToken(crate::xml::errors::Error),
    UnterminatedEscapeChar(String),
    UnknownEscapeChar(String),
    Utf8Error(std::string::FromUtf8Error),
    RequestError(ureq::Error),
    MissingVersion,
    MissingChannel,
    MissingLink,
    UnexpectedStatus(ureq::http::StatusCode),
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
            Self::MissingVersion => write!(f, "Feed is missing a version attribute"),
            Self::MissingChannel => write!(f, "Feed is missing a <channel> element"),
            Self::MissingLink => write!(f, "Item has no link to follow"),
            Self::UnexpectedStatus(status) => write!(f, "Unexpected HTTP status: {}", status),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::XmlToken(err) => Some(err),
            Self::Utf8Error(err) => Some(err),
            Self::RequestError(err) => Some(err),
            _ => None,
        }
    }
}

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

#[derive(Clone, Copy)]
enum RssElement {
    Rss,
    Channel,
    Item,
}

#[derive(Clone, Copy)]
enum RssTag {
    Title,
    Link,
    Description,
    Author,
    Language,
}

#[derive(Debug, Default, Clone)]
pub struct RssItem<'a> {
    pub title: Option<Cow<'a, str>>,
    pub link: Option<Cow<'a, str>>,
    pub description: Option<Cow<'a, str>>,
    pub author: Option<Cow<'a, str>>,
}

#[derive(Debug, Default, Clone)]
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
    // TODO: Incorporate other optional fields (pubDate, category, etc).
}

impl<'a> Display for RssItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = self.title.as_deref().unwrap_or("Untitled");
        write!(f, "Title: {}", title)?;

        if let Some(link) = self.link.as_deref() {
            write!(f, "\nLink: {}", link)?;
        }

        if let Some(author) = self.author.as_deref() {
            write!(f, "\nAuthor: {}", author)?;
        }

        if let Some(description) = self.description.as_deref() {
            write!(f, "\nDescription: {}", description)?;
        }

        writeln!(f)
    }
}

/// Unescape the five predefined XML entities plus numeric character references
/// (`&#39;`, `&#x27;`, ...). Returns a borrowed `Cow` when no escaping was needed,
/// avoiding an allocation for the common case.
///
/// Fails on an unterminated `&...` sequence (no closing `;`) or an unrecognized
/// named entity.
fn unescape_xml_control_char(input: &str) -> Result<Cow<'_, str>, Error> {
    if !input.contains('&') {
        return Ok(Cow::Borrowed(input));
    }

    let bytes = input.as_bytes();
    let mut result = Vec::with_capacity(input.len());

    let mut i = 0;
    while i < input.len() {
        if bytes[i] == b'&' {
            let current_idx = i;
            while i < input.len() && bytes[i] != b';' {
                i += 1;
            }

            if i >= input.len() {
                return Err(Error::UnterminatedEscapeChar(
                    String::from_utf8_lossy(&bytes[current_idx..]).into(),
                ));
            }

            let entity = &bytes[current_idx..=i];

            if let Some(numeric) = entity
                .strip_prefix(b"&#")
                .and_then(|rest| rest.strip_suffix(b";"))
            {
                let code_point = if let Some(hex) = numeric
                    .strip_prefix(b"x")
                    .or_else(|| numeric.strip_prefix(b"X"))
                {
                    std::str::from_utf8(hex)
                        .ok()
                        .and_then(|s| u32::from_str_radix(s, 16).ok())
                } else {
                    std::str::from_utf8(numeric)
                        .ok()
                        .and_then(|s| s.parse::<u32>().ok())
                };

                match code_point.and_then(char::from_u32) {
                    Some(ch) => {
                        let mut buf = [0u8; 4];
                        result.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                    }
                    None => {
                        return Err(Error::UnknownEscapeChar(
                            String::from_utf8_lossy(entity).into(),
                        ));
                    }
                }
            } else {
                let unescaped = match entity {
                    b"&lt;" => b'<',
                    b"&gt;" => b'>',
                    b"&quot;" => b'"',
                    b"&apos;" => b'\'',
                    b"&amp;" => b'&',
                    _ => {
                        return Err(Error::UnknownEscapeChar(
                            String::from_utf8_lossy(entity).into(),
                        ));
                    }
                };
                result.push(unescaped);
            }
        } else {
            result.push(bytes[i]);
        }
        i += 1;
    }

    let s = String::from_utf8(result)?;
    Ok(Cow::Owned(s))
}

impl<'a> RssItem<'a> {
    /// Fetch the body at this item's link.
    ///
    /// Returns `Error::MissingLink` if the item has no link, or
    /// `Error::UnexpectedStatus` if the server does not respond with 200 OK.
    pub fn follow_link(&self) -> Result<String, Error> {
        let link = self.link.as_deref().ok_or(Error::MissingLink)?;

        let mut response = ureq::get(link).call()?;

        if response.status() != ureq::http::StatusCode::OK {
            return Err(Error::UnexpectedStatus(response.status()));
        }

        Ok(response.body_mut().read_to_string()?)
    }
}

/// Applies a decoded text value to whichever (element, tag) pair is currently
/// active. Both `Token::Text` and `Token::CharacterData` funnel through here
/// since they were previously handled with duplicated match arms.
fn apply_text<'a>(
    cow_text: Cow<'a, str>,
    current_element: Option<RssElement>,
    current_tag: Option<RssTag>,
    channel: &mut Option<RssChannel<'a>>,
    current_item: &mut Option<RssItem<'a>>,
) {
    match (current_element, current_tag) {
        (Some(RssElement::Channel), Some(tag)) => {
            if let Some(channel) = channel {
                match tag {
                    RssTag::Title => channel.title = Some(cow_text),
                    RssTag::Link => channel.link = Some(cow_text),
                    RssTag::Description => channel.description = Some(cow_text),
                    RssTag::Language => channel.language = Some(cow_text),
                    RssTag::Author => {} // <author> is not a valid <channel> child; ignore.
                }
            }
        }

        (Some(RssElement::Item), Some(tag)) => {
            if let Some(item) = current_item {
                match tag {
                    RssTag::Title => item.title = Some(cow_text),
                    RssTag::Link => item.link = Some(cow_text),
                    RssTag::Description => item.description = Some(cow_text),
                    RssTag::Author => item.author = Some(cow_text),
                    RssTag::Language => {} // <language> is not a valid <item> child; ignore.
                }
            }
        }

        _ => {}
    }
}

impl<'a> RssFeed<'a> {
    pub fn from_tokenizer(
        tokenizer: &mut crate::xml::tokens::XmlTokenizer<'a>,
    ) -> Result<RssFeed<'a>, Error> {
        let mut feed_version = Option::<&'a str>::None;
        let mut channel = Option::<RssChannel>::None;
        let mut items = Vec::new();

        let mut current_element = Option::<RssElement>::None;
        let mut current_tag = Option::<RssTag>::None;
        let mut current_item = Option::<RssItem>::None;

        for token_result in tokenizer.by_ref() {
            let token = token_result?;

            match token {
                crate::xml::tokens::XmlToken::StartTag("rss") => {
                    current_element = Some(RssElement::Rss);
                }

                crate::xml::tokens::XmlToken::StartTag("channel") => {
                    channel = Some(RssChannel::default());
                    current_element = Some(RssElement::Channel);
                }

                crate::xml::tokens::XmlToken::StartTag("item") => {
                    current_item = Some(RssItem::default());
                    current_element = Some(RssElement::Item);
                }

                crate::xml::tokens::XmlToken::StartTag("title") => {
                    current_tag = Some(RssTag::Title);
                }

                crate::xml::tokens::XmlToken::StartTag("link") => {
                    current_tag = Some(RssTag::Link);
                }

                crate::xml::tokens::XmlToken::StartTag("description") => {
                    current_tag = Some(RssTag::Description);
                }

                crate::xml::tokens::XmlToken::StartTag("author") => {
                    current_tag = Some(RssTag::Author);
                }

                crate::xml::tokens::XmlToken::StartTag("language") => {
                    current_tag = Some(RssTag::Language);
                }

                crate::xml::tokens::XmlToken::EndTag("rss") => {
                    current_element = None;
                }

                crate::xml::tokens::XmlToken::EndTag("channel") => {
                    current_element = Some(RssElement::Rss);
                }

                crate::xml::tokens::XmlToken::EndTag("item") => {
                    if let Some(item) = current_item.take() {
                        items.push(item);
                        current_element = Some(RssElement::Channel);
                    }
                }

                crate::xml::tokens::XmlToken::EndTag(
                    "title" | "link" | "description" | "author" | "language",
                ) => {
                    current_tag = None;
                }

                crate::xml::tokens::XmlToken::Attribute { name, value } => {
                    if let (Some(RssElement::Rss), "version") = (&current_element, name) {
                        feed_version = Some(value);
                    }
                }

                crate::xml::tokens::XmlToken::Text(text)
                | crate::xml::tokens::XmlToken::CharacterData(text) => {
                    let cow_text = unescape_xml_control_char(text)?;
                    apply_text(
                        cow_text,
                        current_element,
                        current_tag,
                        &mut channel,
                        &mut current_item,
                    );
                }

                _ => {}
            }
        }

        Ok(RssFeed {
            version: feed_version
                .map(Cow::Borrowed)
                .ok_or(Error::MissingVersion)?,
            channel: channel.ok_or(Error::MissingChannel)?,
            items,
        })
    }
}
