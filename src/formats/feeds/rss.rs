use std::borrow::Cow;

use crate::formats::xml::tokens::XmlToken;

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
/*
impl<'a> std::fmt::Display for RssParser<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         let feed = self.feed.expect("Nothing to display here");

         feed.
     } 
}
*/

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

                XmlToken::StartTag(name) => match name {
                    b"title" => {
                        self.current_tag = Some(RssTag::Title);
                    }
                    b"link" => {
                        self.current_tag = Some(RssTag::Link);
                    }
                    b"description" => {
                        self.current_tag = Some(RssTag::Description);
                    }
                    b"language" => {
                        self.current_tag = Some(RssTag::Language);
                    }
                    b"copyright" => {
                        self.current_tag = Some(RssTag::Copyright);
                    }
                    b"managingEditor" => {
                        self.current_tag = Some(RssTag::ManagingEditor);
                    }
                    b"webMaster" => {
                        self.current_tag = Some(RssTag::WebMaster);
                    }
                    b"pubDate" => {
                        self.current_tag = Some(RssTag::PubDate);
                    }
                    b"lastBuildDate" => {
                        self.current_tag = Some(RssTag::LastBuildDate);
                    }
                    b"category" => {
                        self.current_tag = Some(RssTag::Category);
                    }
                    b"generator" => {
                        self.current_tag = Some(RssTag::Generator);
                    }
                    b"docs" => {
                        self.current_tag = Some(RssTag::Docs);
                    }
                    b"cloud" => {
                        self.current_tag = Some(RssTag::Cloud);
                    }
                    b"ttl" => {
                        self.current_tag = Some(RssTag::Ttl);
                    }
                    b"image" => {
                        self.current_tag = Some(RssTag::Image);
                    }
                    b"rating" => {
                        self.current_tag = Some(RssTag::Rating);
                    }
                    b"textInput" => {
                        self.current_tag = Some(RssTag::TextInput);
                    }
                    b"skipHours" => {
                        self.current_tag = Some(RssTag::SkipHours);
                    }
                    b"skipDays" => {
                        self.current_tag = Some(RssTag::SkipDays);
                    }

                    _ => {}
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

                XmlToken::StartTag(name) => match name {
                    b"title" => {
                        self.current_tag = Some(RssTag::Title);
                    }
                    b"link" => {
                        self.current_tag = Some(RssTag::Link);
                    }
                    b"description" => {
                        self.current_tag = Some(RssTag::Description);
                    }
                    b"author" => {
                        self.current_tag = Some(RssTag::Author);
                    }
                    b"category" => {
                        self.current_tag = Some(RssTag::Category);
                    }
                    b"comments" => {
                        self.current_tag = Some(RssTag::Comments);
                    }
                    b"enclosure" => {
                        self.current_tag = Some(RssTag::Enclosure);
                    }
                    b"guid" => {
                        self.current_tag = Some(RssTag::Guid);
                    }
                    b"pubDate" => {
                        self.current_tag = Some(RssTag::PubDate);
                    }
                    b"source" => {
                        self.current_tag = Some(RssTag::Source);
                    }

                    _ => {}
                },

                XmlToken::EndTag(name) => match name {
                    b"title" => {
                        self.current_tag = None;
                    }
                    b"link" => {
                        self.current_tag = None;
                    }
                    b"description" => {
                        self.current_tag = None;
                    }
                    b"author" => {
                        self.current_tag = None;
                    }
                    b"category" => {
                        self.current_tag = None;
                    }
                    b"comments" => {
                        self.current_tag = None;
                    }
                    b"enclosure" => {
                        self.current_tag = None;
                    }
                    b"guid" => {
                        self.current_tag = None;
                    }
                    b"pubDate" => {
                        self.current_tag = None;
                    }
                    b"source" => {
                        self.current_tag = None;
                    }
                    _ => {}
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
                            Some(RssTag::Author) => {
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
