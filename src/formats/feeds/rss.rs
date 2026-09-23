// FIXME: current_tag should be a stack and not a single value.

// FIXME: dc / atom namespaces are defined in the xml file,
// do not assume that is will always be dc or atom. This
// should be parsed dynamically.

use std::borrow::Cow;

use crate::formats::{extensions::dublin_core::{DublinCoreLegacyNamespace, DublinCoreParseError}, xml::tokens::{XmlToken, XmlTokenizer}};

#[repr(u8)]
#[derive(Debug)]
pub enum Error {
    Utf8Parse(std::str::Utf8Error),
    Other(String),

    ElementMismatch { expected: RssElement, found: RssElement },
    UnexpectedRssChannelElement(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8Parse(e) => write!(f, "{e}"),
            Self::Other(e) => write!(f, "{e}"),

            Self::ElementMismatch { expected, found }=> write!(f, "Element mismatch. Expected: {:?}, found: {:?}", expected, found),
            Self::UnexpectedRssChannelElement(element) => write!(f, "Unexpected rss channel element: {element}"),

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
enum RssParseState {
    Declaration,
    Feed,
    Channel,
    Item,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
enum RssElement {
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

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
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

impl<'a> TryFrom<&'a [u8]> for RssElement {
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

#[derive(Debug)]
pub struct RssImageElement<'a> {
    url: Option<Cow<'a, str>>,
    title: Option<Cow<'a, str>>,
    link: Option<Cow<'a, str>>,
    //width: Option<Cow<'a, str>>,
    //height: Option<Cow<'a, str>>,
    //description: Option<Cow<'a, str>>,
}

#[derive(Debug)]
pub struct RssCloudElement<'a>(Cow<'a, str>);

#[derive(Debug)]
pub struct RssTtlElement<'a>(Cow<'a, str>);

#[derive(Debug)]
pub struct RssTextInputElement<'a> {
    title: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
    name: Option<Cow<'a, str>>,
    link: Option<Cow<'a, str>>,
}

#[derive(Debug, Default)]
pub struct RssChannel<'a> {
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
    cloud: Option<RssCloudElement<'a>>,
    ttl: Option<RssTtlElement<'a>>,
    image: Option<RssImageElement<'a>>,
    rating: Option<Cow<'a, str>>,
    text_input: Option<RssTextInputElement<'a>>,
    skip_hours: Option<Cow<'a, str>>,
    skip_days: Option<Cow<'a, str>>,
}

#[derive(Debug)]
struct ExtensionElement<'a> {
    prefix: Cow<'a, str>,
    local_name: Cow<'a, str>,
    attributes: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    text: Option<Cow<'a, str>>,
    children: Vec<ExtensionElement<'a>>,
}

#[derive(Debug)]
pub struct RssItemSource<'a> {
    url: Cow<'a, str>
}

#[derive(Debug)]
pub struct RssItemEnclosure<'a> {
    url: Cow<'a, str>,
    length: Cow<'a, str>,
    enclosure_type: Cow<'a, str>
}

#[derive(Debug)]
pub struct RssItemCategory<'a> {
    domain: Option<Cow<'a, str>>,
    value: Cow<'a, str>
}

#[derive(Debug)]
pub struct RssItemGuid<'a> {
    is_perma_link: Cow<'a, str>,
    value: Cow<'a, str>,
}

#[derive(Debug, Default)]
pub struct RssItem<'a> {
    title: Option<Cow<'a, str>>,
    link: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
    author: Option<Cow<'a, str>>,
    category: Option<RssItemCategory<'a>>,
    comments: Option<Cow<'a, str>>,
    enclosure: Option<RssItemEnclosure<'a>>,
    guid: Option<RssItemGuid<'a>>,
    pub_date: Option<Cow<'a, str>>,
    source: Option<RssItemSource<'a>>,
}

impl<'a> std::fmt::Display for RssChannel<'a> {
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
            writeln!(f, "Cloud: {:?}", cloud)?;
        }
        
        if let Some(ttl) = &self.ttl{
            writeln!(f, "Ttl: {:?}", ttl)?;
        }

        if let Some(image) = &self.image {
            writeln!(f, "Image: {:?}", image)?;
        }
        
        if let Some(rating) = &self.rating{
            writeln!(f, "Rating: {}", rating)?;
        }

        if let Some(text_input) = &self.text_input{
            writeln!(f, "Text input: {:?}", text_input)?;
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

impl<'a> std::fmt::Display for RssItem<'a> {
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
            writeln!(f, "Category: {:?}", category)?;
        }

        if let Some(comments) = &self.comments{
            writeln!(f, "Comments: {}", comments)?;
        }
        
        if let Some(enclosure) = &self.enclosure{
            writeln!(f, "Enclosure: {:?}", enclosure)?;
        }

        if let Some(guid) = &self.guid{
            writeln!(f, "Guid: {:?}", guid)?;
        }

        if let Some(pub_date) = &self.pub_date{
            writeln!(f, "Pub date: {}", pub_date)?;
        }

        if let Some(source) = &self.source{
            writeln!(f, "Source: {:?}", source)?;
        }
        
        Ok(())
    }
}



pub struct RssFeedParser<'a> {
    tokenizer: XmlTokenizer<'a>,
    open_elements: Vec<RssElement>,
    pub channel: RssChannel<'a>,
    pub items: Vec<RssItem<'a>>,
}

impl<'a> From<XmlTokenizer<'a>> for RssFeedParser<'a> {
    fn from(value: XmlTokenizer<'a>) -> Self {
        Self {
            tokenizer: value,
            open_elements: Vec::new(),
            channel: RssChannel::default(),
            items: Vec::new()
        }
    }
}

impl<'a> RssFeedParser<'a> {

    #[inline]
    fn parse_str(bytes: &'a [u8]) -> Result<Cow<'a, str>, Error> {
        Ok(std::str::from_utf8(bytes).map(Cow::Borrowed)?)
    }

    pub fn parse_rss_feed_from_tokenizer(&mut self) -> Result<(), Error> {

        
        for token in self.tokenizer.by_ref() {

            match token {
                Ok(XmlToken::Declaration(_)) => {},
                Ok(XmlToken::DeclarationTagEnd) => {},

                Ok(XmlToken::StartTag(b"rss")) => {},
                
                Ok(XmlToken::StartTag(b"channel")) => {},

                Ok(XmlToken::StartTag(b"item")) => {
                    self.items.push(RssItem::default());
                },

                Ok(XmlToken::StartTag(name)) => {
                    let element = RssElement::try_from(name)?;
                    self.open_elements.push(element);
                },
                
                Ok(XmlToken::TagEnd { self_closing }) if self_closing => {
                    self.open_elements.pop();
                },

                Ok(XmlToken::EndTag(name)) => {
                    if let Some(previous_element) = self.open_elements.last() {
                        let current_element = RssElement::try_from(name)?;
                        if &current_element == previous_element {
                            self.open_elements.pop();
                            continue;
                        }
                        return Err(Error::ElementMismatch { expected: *previous_element, found: current_element });
                    }
                    
                }

                Ok(XmlToken::Text(data)) | Ok(XmlToken::CharacterData(data)) => {

                    let parsed_data = RssFeedParser::parse_str(data)?;
                    
                    // If items are empty, mutate channel.
                    if self.items.is_empty() {
                        match self.open_elements.last() {
                            Some(RssElement::Title) => self.channel.title = Some(parsed_data),
                            Some(RssElement::Link) => self.channel.link = Some(parsed_data),
                            Some(RssElement::Description) => self.channel.description = Some(parsed_data),
                            Some(RssElement::Language) => self.channel.language = Some(parsed_data),
                            Some(RssElement::Copyright) => self.channel.copyright = Some(parsed_data),
                            Some(RssElement::ManagingEditor) => self.channel.managing_editor = Some(parsed_data),
                            Some(RssElement::WebMaster) => self.channel.web_master = Some(parsed_data),
                            Some(RssElement::PubDate) => self.channel.pub_date = Some(parsed_data),
                            Some(RssElement::LastBuildDate) => self.channel.last_build_date = Some(parsed_data),
                            Some(RssElement::Category) => self.channel.category = Some(parsed_data),
                            Some(RssElement::Generator) => self.channel.generator = Some(parsed_data),
                            Some(RssElement::Docs) => self.channel.docs = Some(parsed_data),
                            //Some(RssElement::Cloud) => self.channel.cloud = Some(parsed_data),
                            //Some(RssElement::Ttl) => self.channel.ttl = Some(parsed_data),
                            //Some(RssElement::Image) => self.channel.image = Some(parsed_data),
                            Some(RssElement::Rating) => self.channel.rating = Some(parsed_data),
                            //Some(RssElement::TextInput) => self.channel.text_input = Some(parsed_data),
                            Some(RssElement::SkipHours) => self.channel.skip_hours = Some(parsed_data),
                            Some(RssElement::SkipDays) => self.channel.skip_days = Some(parsed_data),

                            //_ => return Err(Error::UnexpectedRssChannelElement(parsed_data.to_string())),
                            _ => {}
                        }

                    } else {
                        if let Some(current_item) = self.items.last_mut() {
                            match self.open_elements.last() {
                                Some(RssElement::Title) => current_item.title = Some(parsed_data),
                                Some(RssElement::Link) => current_item.link = Some(parsed_data),
                                Some(RssElement::Description) => current_item.description = Some(parsed_data),
                                Some(RssElement::Author) | Some(RssElement::DublinCore(DublinCoreLegacyNamespace::Creator)) => current_item.author = Some(parsed_data),
                                //Some(RssElement::Category) => current_item.category = Some(parsed_data),
                                Some(RssElement::Comments) => current_item.comments = Some(parsed_data),
                                //Some(RssElement::Enclosure) => current_item.enclosure= Some(parsed_data),
                                //Some(RssElement::Guid) => current_item.guid = Some(parsed_data),
                                Some(RssElement::PubDate) => current_item.pub_date= Some(parsed_data),
                                //Some(RssElement::Source) => current_item.source = Some(parsed_data),

                                _ => {},
                            }
                        }


                    }

                },

                Ok(XmlToken::Attribute { name, value }) => {

                }

                //_ => unimplemented!("parse_rss_feed_from_tokenizer")
                _ => {}



            }



       } 

        Ok(())
    }
}










/*
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


#[derive(Debug, Default)]
pub struct Feed<'a> {
    version: Option<Cow<'a, str>>,
    namespaces: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    pub channel: Option<RssChannel<'a>>,
    pub items: Vec<RssItem<'a>>,
}

#[derive(Debug)]
pub struct RssParser<'a> {
    state: RssParseState,
    pub feed: Option<Feed<'a>>,
    current_tag: Option<RssElement>,
}

impl<'a> Default for RssParser<'a> {
    fn default() -> Self {
        Self {
            state: RssParseState::Declaration,
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
            RssParseState::Declaration => {
                // For now, skip everything until the rss start tag is found.
                if let XmlToken::StartTag(name) = token
                    && name == b"rss"
                {
                    self.state = RssParseState::Feed;
                    self.feed = Some(Feed::default());
                }
            }

            RssParseState::Feed => match token {
                XmlToken::StartTag(name) if name == b"channel" => {
                    self.state = RssParseState::Channel;
                    if let Some(feed) = self.feed.as_mut() {
                        feed.channel = Some(RssChannel::default());
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

            RssParseState::Channel => match token {
                XmlToken::StartTag(name) if name == b"item" => {
                    self.state = RssParseState::Item;
                    if let Some(feed) = self.feed.as_mut() {
                        feed.items.push(RssItem::default());
                    }
                }

                XmlToken::EndTag(name) if name == b"channel" => {
                    self.state = RssParseState::Feed;
                }

                XmlToken::StartTag(name) => {
                    self.current_tag = Some(RssElement::try_from(name)?);
                },

                XmlToken::EndTag(_) => {
                    self.current_tag = None;
                }

                XmlToken::Text(text) => {
                    let parsed_text = Self::parse_str(text)?;

                    if let Some(channel) = self.feed.as_mut().and_then(|f| f.channel.as_mut()) {
                        match self.current_tag {
                            Some(RssElement::Title) => {
                                channel.title = Some(parsed_text);
                            }
                            Some(RssElement::Link) => {
                                channel.link = Some(parsed_text);
                            }
                            Some(RssElement::Description) => {
                                channel.description = Some(parsed_text);
                            }
                            Some(RssElement::Language) => {
                                channel.language = Some(parsed_text);
                            }
                            Some(RssElement::Copyright) => {
                                channel.copyright = Some(parsed_text);
                            }
                            Some(RssElement::ManagingEditor) => {
                                channel.managing_editor = Some(parsed_text);
                            }
                            Some(RssElement::WebMaster) => {
                                channel.web_master = Some(parsed_text);
                            }
                            Some(RssElement::PubDate) => {
                                channel.pub_date = Some(parsed_text);
                            }
                            Some(RssElement::LastBuildDate) => {
                                channel.last_build_date = Some(parsed_text);
                            }
                            Some(RssElement::Category) => {
                                channel.category = Some(parsed_text);
                            }
                            Some(RssElement::Generator) => {
                                channel.generator = Some(parsed_text);
                            }
                            Some(RssElement::Docs) => {
                                channel.docs = Some(parsed_text);
                            }
                            Some(RssElement::Cloud) => {
                                //channel.cloud= Some(parsed_text);
                            }
                            Some(RssElement::Ttl) => {
                                //channel.ttl = Some(parsed_text);
                            }
                            Some(RssElement::Image) => {
                                //channel.image = Some(parsed_text);
                            }
                            Some(RssElement::Rating) => {
                                channel.rating = Some(parsed_text);
                            }
                            Some(RssElement::TextInput) => {
                                //channel.text_input = Some(parsed_text);
                            }
                            Some(RssElement::SkipHours) => {
                                channel.skip_hours = Some(parsed_text);
                            }
                            Some(RssElement::SkipDays) => {
                                channel.skip_days = Some(parsed_text);
                            }

                            _ => {}
                        }
                    }
                }

                _ => {}
            },

            RssParseState::Item => match token {
                XmlToken::EndTag(name) if name == b"item" => {
                    self.state = RssParseState::Channel;
                }

                XmlToken::StartTag(name) => {
                    self.current_tag = Some(RssElement::try_from(name)?);
                },

                XmlToken::EndTag(_) => {
                    self.current_tag = None;
                },

                XmlToken::Text(text) | XmlToken::CharacterData(text) => {
                    let parsed_text = Self::parse_str(text)?;

                    if let Some(item) = self.feed.as_mut().and_then(|f| f.items.last_mut()) {
                        match self.current_tag {
                            Some(RssElement::Title) => {
                                item.title = Some(parsed_text);
                            }
                            Some(RssElement::Link) => {
                                item.link = Some(parsed_text);
                            }
                            Some(RssElement::Description) => {
                                item.description = Some(parsed_text);
                            }
                            Some(RssElement::Author) | Some(RssElement::DublinCore(DublinCoreLegacyNamespace::Creator)) => {
                                item.author = Some(parsed_text);
                            }
                            Some(RssElement::Category) => {
                                item.category = Some(parsed_text);
                            }
                            Some(RssElement::Comments) => {
                                item.comments = Some(parsed_text);
                            }
                            Some(RssElement::Enclosure) => {
                                item.enclosure = Some(parsed_text);
                            }
                            Some(RssElement::Guid) => {
                                item.guid = Some(parsed_text);
                            }
                            Some(RssElement::PubDate) => {
                                item.pub_date = Some(parsed_text);
                            }
                            Some(RssElement::Source) => {
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

*/
