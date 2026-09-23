use std::borrow::Cow;

use crate::formats::{extensions::dublin_core::{DublinCoreLegacyNamespace, DublinCoreParseError}, xml::tokens::XmlToken};

#[repr(u8)]
#[derive(Debug)]
pub enum Error {
    Utf8Parse(std::str::Utf8Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8Parse(e) => write!(f, "{e}"),
            Self::Other(e) => write!(f, "{e}")

        }
    }
}

impl std::error::Error for Error {}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8Parse(value)
    }
}

// FIXME: remove this allow in the future.
#[allow(unused_variables)]
impl From<DublinCoreParseError> for Error {
    fn from(value: DublinCoreParseError) -> Self {
        Self::Other("Byte parse error".into())
    }
}


#[derive(Debug, PartialEq, Eq)]
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
    Language,
    Copyright,
    ManagingEditor,
    WebMaster,
    PubDate,
    LastBuildDate,
    Category,
    Generator,
    Docs,
    Cloud,
    Ttl,
    Image,
    Rating,
    TextInput,
    SkipHours,
    SkipDays,

    Author,
    Comments,
    Enclosure,
    Guid,
    Source,
    
    Atom(AtomTag),
    DublinCore(DublinCoreLegacyNamespace),
}

#[derive(Debug)]
enum AtomTag {
    Link
}

impl<'a> TryFrom<&'a [u8]> for AtomTag {
    type Error = Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value {
            b"atom:link" => Ok(Self::Link),

            _ => Err(Error::Other(format!("Unable to parse atom tag: {}", String::from_utf8_lossy(value)))),
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for RssTag {
    type Error = Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value {
            b"title" => Ok(Self::Title),
            b"link" => Ok(Self::Link),
            b"description" => Ok(Self::Description),
            b"language" => Ok(Self::Language),
            b"copyright" => Ok(Self::Copyright),
            b"managingEditor" => Ok(Self::ManagingEditor),
            b"webMaster" => Ok(Self::WebMaster),
            b"pubDate" => Ok(Self::PubDate),
            b"lastBuildDate" => Ok(Self::LastBuildDate),
            b"category" => Ok(Self::Category),
            b"generator" => Ok(Self::Generator),
            b"docs" => Ok(Self::Docs),
            b"cloud" => Ok(Self::Cloud),
            b"ttl" => Ok(Self::Ttl),
            b"image" => Ok(Self::Image),
            b"rating" => Ok(Self::Rating),
            b"textInput" => Ok(Self::TextInput),
            b"skipHours" => Ok(Self::SkipHours),
            b"skipDays" => Ok(Self::SkipDays),
            b"author" => Ok(Self::Author),
            b"comments" => Ok(Self::Comments),
            b"enclosure" => Ok(Self::Enclosure),
            b"guid" => Ok(Self::Guid),
            b"source" => Ok(Self::Source),
            _ if value.starts_with(b"dc:") => {
                Ok(Self::DublinCore(DublinCoreLegacyNamespace::try_from(value)?))
            },
            _ if value.starts_with(b"atom:") => {
                Ok(Self::Atom(AtomTag::try_from(value)?))
            }
            _ => Err(Error::Other(format!("Unable to parse rss tag: {}", String::from_utf8_lossy(value)))),
             
         } 
    }
    
}



#[derive(Debug, Default)]
pub struct Channel<'a> {
    title: Option<Cow<'a, str>>,
    link: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
    language: Option<Cow<'a, str>>,
    copyright: Option<Cow<'a, str>>,
    managing_editor: Option<Cow<'a, str>>,
    web_master: Option<Cow<'a, str>>,
    pub_date: Option<Cow<'a, str>>,
    last_build_date: Option<Cow<'a, str>>,
    category: Option<Cow<'a, str>>,
    generator: Option<Cow<'a, str>>,
    docs: Option<Cow<'a, str>>,
    cloud: Option<Cow<'a, str>>,
    ttl: Option<Cow<'a, str>>,
    image: Option<Cow<'a, str>>,
    rating: Option<Cow<'a, str>>,
    text_input: Option<Cow<'a, str>>,
    skip_hours: Option<Cow<'a, str>>,
    skip_days: Option<Cow<'a, str>>,
}

#[derive(Debug)]
struct ExtensionField<'a> {
    prefix: Cow<'a, str>,
    local_name: Cow<'a, str>,
    value: Cow<'a, str>,
}

#[derive(Debug, Default)]
pub struct Item<'a> {
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
    pub channel: Option<Channel<'a>>,
    pub items: Vec<Item<'a>>,
}



impl<'a> std::fmt::Display for Channel<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Channel\n")?;

        if let Some(title) = &self.title {
            writeln!(f, "Title: {}", title)?;
        }

        if let Some(link) = &self.link {
            writeln!(f, "Link: {}", link)?;
        }

        if let Some(description) = &self.description {
            writeln!(f, "Description: {}", description)?;
        }

        if let Some(language) = &self.language{
            writeln!(f, "Language: {}", language)?;
        }

        if let Some(copyright) = &self.copyright{
            writeln!(f, "Copyright: {}", copyright)?;
        }

        if let Some(managing_editor) = &self.managing_editor{
            writeln!(f, "Managing editor: {}", managing_editor)?;
        }

        if let Some(web_master) = &self.web_master{
            writeln!(f, "Web master: {}", web_master)?;
        }

        if let Some(pub_date) = &self.pub_date{
            writeln!(f, "Pub date: {}", pub_date)?;
        }

        if let Some(last_build_date) = &self.last_build_date{
            writeln!(f, "Last build date: {}", last_build_date)?;
        }

        if let Some(category) = &self.category{
            writeln!(f, "Category: {}", category)?;
        }

        if let Some(generator) = &self.generator{
            writeln!(f, "Generator: {}", generator)?;
        }

        if let Some(docs) = &self.docs{
            writeln!(f, "Docs: {}", docs)?;
        }

        if let Some(cloud) = &self.cloud{
            writeln!(f, "Cloud: {}", cloud)?;
        }
        
        if let Some(ttl) = &self.ttl{
            writeln!(f, "Ttl: {}", ttl)?;
        }

        if let Some(image) = &self.image {
            writeln!(f, "Image: {}", image)?;
        }
        
        if let Some(rating) = &self.rating{
            writeln!(f, "Rating: {}", rating)?;
        }

        if let Some(text_input) = &self.text_input{
            writeln!(f, "Text input: {}", text_input)?;
        }

        if let Some(skip_hours) = &self.skip_hours{
            writeln!(f, "Skip hours: {}", skip_hours)?;
        }

        if let Some(skip_days) = &self.skip_days{
            writeln!(f, "Skip days: {}", skip_days)?;
        }

        Ok(())

    }
}

impl<'a> std::fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Item\n")?;

        if let Some(title) = &self.title {
            writeln!(f, "Title: {}", title)?;
        }

        if let Some(link) = &self.link{
            writeln!(f, "Link: {}", link)?;
        }

        if let Some(description) = &self.description{
            writeln!(f, "Description: {}", description)?;
        }

        if let Some(author) = &self.author{
            writeln!(f, "Author: {}", author)?;
        }

        if let Some(category) = &self.category{
            writeln!(f, "Category: {}", category)?;
        }

        if let Some(comments) = &self.comments{
            writeln!(f, "Comments: {}", comments)?;
        }
        
        if let Some(enclosure) = &self.enclosure{
            writeln!(f, "Enclosure: {}", enclosure)?;
        }

        if let Some(guid) = &self.guid{
            writeln!(f, "Guid: {}", guid)?;
        }

        if let Some(pub_date) = &self.pub_date{
            writeln!(f, "Pub date: {}", pub_date)?;
        }

        if let Some(source) = &self.source{
            writeln!(f, "Source: {}", source)?;
        }
        
        Ok(())
    }
}

impl<'a> Feed<'a> {
    pub fn get_channel_title(&self) -> Option<&str> {
        self.channel.as_ref().and_then(|c| c.title.as_deref())
    }

    pub fn get_channel_link(&self) -> Option<&str> {
        self.channel.as_ref().and_then(|c| c.link.as_deref())
    }

    pub fn get_item_titles(&self) -> Vec<&str> {
        self.items
            .iter()
            .filter_map(|x| x.title.as_deref())
            .collect()
    }

    pub fn get_item_descriptions(&self) -> Vec<&str> {
        self.items
            .iter()
            .filter_map(|x| x.description.as_deref())
            .collect()
    }
}

#[derive(Debug)]
pub struct RssParser<'a> {
    state: XmlParseState,
    pub feed: Option<Feed<'a>>,
    current_tag: Option<RssTag>,
}

impl<'a> Default for RssParser<'a> {
    fn default() -> Self {
        Self {
            state: XmlParseState::Declaration,
            feed: None,
            current_tag: None,
        }
    }
}

impl<'a> RssParser<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn parse_str(bytes: &'a [u8]) -> Result<Cow<'a, str>, Error> {
        Ok(std::str::from_utf8(bytes).map(Cow::Borrowed)?)
    }

    pub fn handle_token(&mut self, token: XmlToken<'a>) -> Result<(), Error> {
        match self.state {
            XmlParseState::Declaration => {
                // For now, skip everything until the rss start tag is found.
                if let XmlToken::StartTag(name) = token
                    && name == b"rss"
                {
                    self.state = XmlParseState::Feed;
                    self.feed = Some(Feed::default());
                }
            }

            XmlParseState::Feed => match token {
                XmlToken::StartTag(name) if name == b"channel" => {
                    self.state = XmlParseState::Channel;
                    if let Some(feed) = self.feed.as_mut() {
                        feed.channel = Some(Channel::default());
                    }
                }

                XmlToken::EndTag(name) if name == b"rss" => {}

                XmlToken::Attribute { name, value } if name == b"version" => {
                    if let Some(feed) = self.feed.as_mut() {
                        feed.version = Some(Self::parse_str(value)?);
                    }
                }

                XmlToken::Attribute { name, value } => {
                    if let Some(feed) = self.feed.as_mut() {
                        let name = Self::parse_str(name)?;
                        let value = Self::parse_str(value)?;
                        feed.namespaces.push((name, value));
                    }
                }

                _ => {}
            },

            XmlParseState::Channel => match token {
                XmlToken::StartTag(name) if name == b"item" => {
                    self.state = XmlParseState::Item;
                    if let Some(feed) = self.feed.as_mut() {
                        feed.items.push(Item::default());
                    }
                }

                XmlToken::EndTag(name) if name == b"channel" => {
                    self.state = XmlParseState::Feed;
                }

                XmlToken::StartTag(name) => {
                    self.current_tag = Some(RssTag::try_from(name)?);
                },

                XmlToken::EndTag(_) => {
                    self.current_tag = None;
                }

                XmlToken::Text(text) => {
                    let parsed_text = Self::parse_str(text)?;

                    if let Some(channel) = self.feed.as_mut().and_then(|f| f.channel.as_mut()) {
                        match self.current_tag {
                            Some(RssTag::Title) => {
                                channel.title = Some(parsed_text);
                            }
                            Some(RssTag::Link) => {
                                channel.link = Some(parsed_text);
                            }
                            Some(RssTag::Description) => {
                                channel.description = Some(parsed_text);
                            }
                            Some(RssTag::Language) => {
                                channel.language = Some(parsed_text);
                            }
                            Some(RssTag::Copyright) => {
                                channel.copyright = Some(parsed_text);
                            }
                            Some(RssTag::ManagingEditor) => {
                                channel.managing_editor = Some(parsed_text);
                            }
                            Some(RssTag::WebMaster) => {
                                channel.web_master = Some(parsed_text);
                            }
                            Some(RssTag::PubDate) => {
                                channel.pub_date = Some(parsed_text);
                            }
                            Some(RssTag::LastBuildDate) => {
                                channel.last_build_date = Some(parsed_text);
                            }
                            Some(RssTag::Category) => {
                                channel.category = Some(parsed_text);
                            }
                            Some(RssTag::Generator) => {
                                channel.generator = Some(parsed_text);
                            }
                            Some(RssTag::Docs) => {
                                channel.docs = Some(parsed_text);
                            }
                            Some(RssTag::Cloud) => {
                                channel.cloud= Some(parsed_text);
                            }
                            Some(RssTag::Ttl) => {
                                channel.ttl = Some(parsed_text);
                            }
                            Some(RssTag::Image) => {
                                channel.image = Some(parsed_text);
                            }
                            Some(RssTag::Rating) => {
                                channel.rating = Some(parsed_text);
                            }
                            Some(RssTag::TextInput) => {
                                channel.text_input = Some(parsed_text);
                            }
                            Some(RssTag::SkipHours) => {
                                channel.skip_hours = Some(parsed_text);
                            }
                            Some(RssTag::SkipDays) => {
                                channel.skip_days = Some(parsed_text);
                            }

                            _ => {}
                        }
                    }
                }

                _ => {}
            },

            XmlParseState::Item => match token {
                XmlToken::EndTag(name) if name == b"item" => {
                    self.state = XmlParseState::Channel;
                }

                XmlToken::StartTag(name) => {
                    self.current_tag = Some(RssTag::try_from(name)?);
                },

                XmlToken::EndTag(_) => {
                    self.current_tag = None;
                },

                XmlToken::Text(text) | XmlToken::CharacterData(text) => {
                    let parsed_text = Self::parse_str(text)?;

                    if let Some(item) = self.feed.as_mut().and_then(|f| f.items.last_mut()) {
                        match self.current_tag {
                            Some(RssTag::Title) => {
                                item.title = Some(parsed_text);
                            }
                            Some(RssTag::Link) => {
                                item.link = Some(parsed_text);
                            }
                            Some(RssTag::Description) => {
                                item.description = Some(parsed_text);
                            }
                            Some(RssTag::Author) | Some(RssTag::DublinCore(DublinCoreLegacyNamespace::Creator)) => {
                                item.author = Some(parsed_text);
                            }
                            Some(RssTag::Category) => {
                                item.category = Some(parsed_text);
                            }
                            Some(RssTag::Comments) => {
                                item.comments = Some(parsed_text);
                            }
                            Some(RssTag::Enclosure) => {
                                item.enclosure = Some(parsed_text);
                            }
                            Some(RssTag::Guid) => {
                                item.guid = Some(parsed_text);
                            }
                            Some(RssTag::PubDate) => {
                                item.pub_date = Some(parsed_text);
                            }
                            Some(RssTag::Source) => {
                                item.source = Some(parsed_text);
                            }
                            _ => {}
                        }
                    }
                }

                _ => {}
            },
        }
        Ok(())
    }
}
