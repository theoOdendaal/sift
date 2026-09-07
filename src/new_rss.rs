use std::borrow::Cow;

use crate::xml::byte_token::XmlToken;

#[derive(Debug)]
#[derive(PartialEq, Eq)]
enum XmlParseState {
    Declaration,
    Feed,
    Channel,
    Item,
}

#[derive(Debug, Default)]
pub struct Channel<'a> {
    title: Option<Cow<'a, str>>,
    link: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
    items: Vec<Item<'a>>,
}

#[derive(Debug, Default)]
pub struct ExtensionField<'a> {
    prefix: Cow<'a, str>,
    local_name: Cow<'a, str>,
    value: Cow<'a, str>,
}

#[derive(Debug, Default)]
pub struct Item<'a> {
    title: Option<Cow<'a, str>>,
    link: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
    author: Option<Cow<'a, str>>,
    category: Option<Cow<'a, str>>,
    comments: Option<Cow<'a, str>>,
    enclosure: Option<Cow<'a, str>>,
    guid: Option<Cow<'a, str>>,
    pub_date: Option<Cow<'a, str>>,
    source: Option<Cow<'a, str>>,
    extensions: Vec<ExtensionField<'a>>,
}

#[derive(Debug, Default)]
pub struct Feed<'a> {
    version: Option<Cow<'a, str>>,
    namespaces: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    channel: Option<Channel<'a>>,
}

#[derive(Debug)]
pub struct RssParser<'a> {
    state: XmlParseState,
    feed: Option<Feed<'a>>,
}

fn bytes_to_cow_borrowed<'a>(bytes: &'a [u8]) -> Result<Cow<'a, str>, std::str::Utf8Error> {
    let s = str::from_utf8(bytes)?;
    Ok(Cow::Borrowed(s))
}

impl<'a> RssParser<'a> {
    pub fn new() -> Self {
        Self {
            state: XmlParseState::Declaration,
            feed: None,
        }
    }

    pub fn handle_token(&mut self, token: XmlToken<'a>) {
        
        match self.state {
            XmlParseState::Declaration => {
                // For now, skip everything until the rss start tag is found.
                if let XmlToken::StartTag(name) = token && name == b"rss" {
                    self.state = XmlParseState::Feed;
                    self.feed = Some(Feed::default());
                }
            },

            XmlParseState::Feed => {
                match token {
                    XmlToken::StartTag(name) if name == b"channel" => {
                        self.state = XmlParseState::Channel;
                        if let Some(feed) = self.feed.as_mut() {
                            feed.channel = Some(Channel::default());
                        }
                    }

                    XmlToken::Attribute { name, value } if name == b"version" => {
                        if let Some(feed) = self.feed.as_mut() {
                            feed.version = Some(bytes_to_cow_borrowed(value).expect("Unable to parse attribute bytes to utf8"));
                        }
                    },

                    XmlToken::Attribute { name, value } => {
                        if let Some(feed) = self.feed.as_mut() {
                            let name = bytes_to_cow_borrowed(name).expect("Unable to parse attribute name to utf8");
                            let value = bytes_to_cow_borrowed(value).expect("Unable to parse attribute name to utf8");
                            feed.namespaces.push((name, value));
                        }
                    }

                    _ => {},
                }
            },

            XmlParseState::Channel => {},

            XmlParseState::Item => {},


        }
    }
}
