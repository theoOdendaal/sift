use std::borrow::Cow;

use crate::xml::tokens::XmlToken;

#[repr(u8)]
#[derive(Debug)]
pub enum Error {
    Utf8Parse(std::str::Utf8Error),

}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8Parse(e) => write!(f, "{}", e),

        }
    }
}

impl std::error::Error for Error {}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8Parse(value)
    }
}


#[derive(Debug)]
#[derive(PartialEq, Eq)]
enum XmlParseState {
    Declaration,
    Feed,
    Channel,
    Item,
}

#[derive(Debug)]
enum RssTag {
    Title,
    Link,
    Description,

    Author,
    Category,
    Comments,
    Enclosure,
    Guid,
    PubDate,
    Source,
}

#[derive(Debug, Default)]
struct Channel<'a> {
    title: Option<Cow<'a, str>>,
    link: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
}

#[derive(Debug)]
struct ExtensionField<'a> {
    prefix: Cow<'a, str>,
    local_name: Cow<'a, str>,
    value: Cow<'a, str>,
}

#[derive(Debug, Default)]
struct Item<'a> {
    extensions: Vec<ExtensionField<'a>>,
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
}

#[derive(Debug, Default)]
pub struct Feed<'a> {
    version: Option<Cow<'a, str>>,
    namespaces: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    channel: Option<Channel<'a>>,
    items: Vec<Item<'a>>,
}

impl<'a> Feed<'a> {
    pub fn get_channel_title(&self) -> Option<Cow<'a, str>> {
        self.channel.as_ref().and_then(|c| c.title.clone())
    }

    pub fn get_item_titles(&self) -> Vec<Cow<'a, str>> {
        self.items.iter().filter_map(|x| x.title.clone()).collect()
    }
}


#[derive(Debug)]
pub struct RssParser<'a> {
    state: XmlParseState,
    pub feed: Option<Feed<'a>>,
    current_tag: Option<RssTag>,
}


impl<'a> RssParser<'a> {
    pub fn new() -> Self {
        Self {
            state: XmlParseState::Declaration,
            feed: None,
            current_tag: None,
        }
    }

    
    #[inline]
    fn parse_str(bytes: &'a [u8]) -> Result<Cow<'a, str>, Error> {
        Ok(std::str::from_utf8(bytes).map(Cow::Borrowed)?)
    }

    pub fn handle_token(&mut self, token: XmlToken<'a>) -> Result<(), Error> {
        
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
                    },

                    XmlToken::EndTag(name) if name == b"rss" => {},


                    XmlToken::Attribute { name, value } if name == b"version" => {
                        if let Some(feed) = self.feed.as_mut() {
                            feed.version = Some(Self::parse_str(value)?);
                        }
                    },

                    XmlToken::Attribute { name, value } => {
                        if let Some(feed) = self.feed.as_mut() {
                            let name = Self::parse_str(name)?;
                            let value = Self::parse_str(value)?;
                            feed.namespaces.push((name, value));
                        }
                    }

                    _ => {},
                }
            },

            XmlParseState::Channel => {
                match token {
                    XmlToken::StartTag(name) if name == b"item" => {
                        self.state = XmlParseState::Item;
                        if let Some(feed) = self.feed.as_mut() {
                            feed.items.push(Item::default());
                        }
                    },

                    XmlToken::EndTag(name) if name == b"channel" => {
                        self.state = XmlParseState::Feed; 
                    },

                    XmlToken::StartTag(name) => {
                        match name {
                            b"title" => { self.current_tag= Some(RssTag::Title); },
                            b"link" => { self.current_tag= Some(RssTag::Link); },
                            b"description" => { self.current_tag= Some(RssTag::Description); },
                            _ => {},
                        }
                    },

                    XmlToken::EndTag(_) => { self.current_tag = None; },

                    XmlToken::Text(text) => {
                        
                        let parsed_text = Self::parse_str(text)?;

                        if let Some(channel) = self.feed.as_mut().and_then(|f| f.channel.as_mut()) {
                            match self.current_tag {
                                Some(RssTag::Title) => { channel.title = Some(parsed_text); },
                                Some(RssTag::Link) => { channel.link= Some(parsed_text); },
                                Some(RssTag::Description) => { channel.description = Some(parsed_text); },
                                _ => {},

                            }
                        } 
                    },

                    _ => {},
                }
            },

            XmlParseState::Item => {
                match token {
                    XmlToken::EndTag(name) if name == b"item" => {
                        self.state = XmlParseState::Channel;
                    },

                    XmlToken::StartTag(name) => {
                        match name {
                            b"title" => { self.current_tag = Some(RssTag::Title); },
                            b"link" => { self.current_tag = Some(RssTag::Link); },
                            b"description" => { self.current_tag = Some(RssTag::Description); },
                            b"author" => { self.current_tag = Some(RssTag::Author); },
                            b"category" => { self.current_tag = Some(RssTag::Category); },
                            b"comments" => { self.current_tag = Some(RssTag::Comments); },
                            b"enclosure" => { self.current_tag = Some(RssTag::Enclosure); },
                            b"guid" => { self.current_tag = Some(RssTag::Guid); },
                            b"pubDate" => { self.current_tag = Some(RssTag::PubDate); },
                            b"source" => { self.current_tag = Some(RssTag::Source); },

                            _ => {},

                        }
                    },
                    
                    XmlToken::EndTag(name) => {
                        match name {
                            b"title" => { self.current_tag = None; },
                            b"link" => { self.current_tag = None; },
                            b"description" => { self.current_tag = None; },
                            b"author" => { self.current_tag = None; },
                            b"category" => { self.current_tag = None; },
                            b"comments" => { self.current_tag = None; },
                            b"enclosure" => { self.current_tag = None; },
                            b"guid" => { self.current_tag = None; },
                            b"pubDate" => { self.current_tag = None; },
                            b"source" => { self.current_tag = None; },
                            _ => {},
                        }
                    },

                    XmlToken::Text(text) | XmlToken::CharacterData(text) => {
                        
                        let parsed_text = Self::parse_str(text)?;

                        if let Some(item) = self.feed.as_mut().and_then(|f| f.items.last_mut()) {
                            match self.current_tag {
                                Some(RssTag::Title) => { item.title = Some(parsed_text); },
                                Some(RssTag::Link) => { item.link = Some(parsed_text); },
                                Some(RssTag::Description) => { item.description = Some(parsed_text); },
                                Some(RssTag::Author) => { item.author = Some(parsed_text); },
                                Some(RssTag::Category) => { item.category = Some(parsed_text); },
                                Some(RssTag::Comments) => { item.comments = Some(parsed_text); },
                                Some(RssTag::Enclosure) => { item.enclosure = Some(parsed_text); },
                                Some(RssTag::Guid) => { item.guid = Some(parsed_text); },
                                Some(RssTag::PubDate) => { item.pub_date = Some(parsed_text); },
                                Some(RssTag::Source) => { item.source = Some(parsed_text); },
                                _ => {}

                            } 
                        }
                    },

                    _ => {}
                }

            },


        }
        Ok(())
    }
}
