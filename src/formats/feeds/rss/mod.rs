pub mod errors;
pub use errors::Error;

// FIXME: dc / atom namespaces are defined in the xml file,
// do not assume that is will always be dc or atom. This
// should be parsed dynamically.

use std::borrow::Cow;

use crate::formats::{extensions::dublin_core::DublinCoreLegacyNamespace, feeds::atom::AtomElement, xml::tokens::{XmlToken, XmlTokenizer}};

#[derive(Debug, Clone, Copy)]
pub enum Namespace {
    Rss,
    Atom,
    DublinCore,
    Content,
    Media,
    Custom,
}

impl Namespace {
    #[inline]
    fn from_uri(uri: &[u8]) -> Self {
        match uri {
            b"http://purl.org/dc/elements/1.1/" => Self::DublinCore,
            b"http://www.w3.org/2005/Atom" => Self::Atom,
            b"http://purl.org/rss/1.0/modules/content/" => Self::Content,
            b"http://search.yahoo.com/mrss/" => Self::Media,
            _ => Self::Custom,
        }
    }
}



#[derive(Debug)]
pub struct NamespaceBinding<'a> {
    prefix: &'a [u8],
    ns: Namespace,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RssElement {
    Rss,
    Channel,
    Item,
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
    Url,
    Width,
    Height,
    Name,
    Atom(AtomElement),
    DublinCore(DublinCoreLegacyNamespace),
    Unknown,
}


impl<'a> From<&'a [u8]> for RssElement {

    fn from(value: &'a [u8]) -> Self {
        match value {
            b"rss" => Self::Rss,
            b"channel" => Self::Channel,
            b"item" => Self::Item,
            b"title" => Self::Title,
            b"link" => Self::Link,
            b"description" => Self::Description,
            b"language" => Self::Language,
            b"copyright" => Self::Copyright,
            b"managingEditor" => Self::ManagingEditor,
            b"webMaster" => Self::WebMaster,
            b"pubDate" => Self::PubDate,
            b"lastBuildDate" => Self::LastBuildDate,
            b"category" => Self::Category,
            b"generator" => Self::Generator,
            b"docs" => Self::Docs,
            b"cloud" => Self::Cloud,
            b"ttl" => Self::Ttl,
            b"image" => Self::Image,
            b"rating" => Self::Rating,
            b"textInput" => Self::TextInput,
            b"skipHours" => Self::SkipHours,
            b"skipDays" => Self::SkipDays,
            b"author" => Self::Author,
            b"comments" => Self::Comments,
            b"enclosure" => Self::Enclosure,
            b"guid" => Self::Guid,
            b"source" => Self::Source,
            b"url" => Self::Url,
            b"width" => Self::Width,
            b"height" => Self::Height,
            b"name" => Self::Name,
            _ => Self::Unknown,
             
         } 
    }
    
}

#[derive(Debug, Default)]
pub struct RssImageElement<'a> {
    url: Option<Cow<'a, str>>,
    title: Option<Cow<'a, str>>,
    link: Option<Cow<'a, str>>,
    width: Option<Cow<'a, str>>,
    height: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
}

#[derive(Debug, Default)]
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
    cloud: Option<Cow<'a, str>>,
    ttl: Option<Cow<'a, str>>,
    image: Option<RssImageElement<'a>>,
    rating: Option<Cow<'a, str>>,
    text_input: Option<RssTextInputElement<'a>>,
    skip_hours: Option<Cow<'a, str>>,
    skip_days: Option<Cow<'a, str>>,
}



#[derive(Debug, Default)]
pub struct RssItemSource<'a> {
    url: Option<Cow<'a, str>>,
    value: Option<Cow<'a, str>>,
}

#[derive(Debug, Default)]
pub struct RssItemEnclosure<'a> {
    url: Option<Cow<'a, str>>,
    length: Option<Cow<'a, str>>,
    enclosure_type: Option<Cow<'a, str>>
}

#[derive(Debug, Default)]
pub struct RssItemCategory<'a> {
    domain: Option<Cow<'a, str>>,
    value: Option<Cow<'a, str>>,
}

#[derive(Debug, Default)]
pub struct RssItemGuid<'a> {
    is_perma_link: Option<Cow<'a, str>>,
    value: Option<Cow<'a, str>>,
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

// open_element_stack and namespace_stack should
// always have exactly the same len. namespace_stack
// represents a collection of all namespaces
// available for use.

pub struct RssFeedParser<'a> {
    tokenizer: XmlTokenizer<'a>,
    open_element_stack: Vec<(RssElement, &'a [u8])>,
    namespace_stack: Vec<Vec<NamespaceBinding<'a>>>,
    pub channel: RssChannel<'a>,
    pub items: Vec<RssItem<'a>>,
}

impl<'a> From<XmlTokenizer<'a>> for RssFeedParser<'a> {
    fn from(value: XmlTokenizer<'a>) -> Self {
        Self {
            tokenizer: value,
            open_element_stack: Vec::new(),
            namespace_stack: Vec::new(),
            channel: RssChannel::default(),
            items: Vec::new()
        }
    }
}

struct PendingTag<'a> {
    raw_name: &'a [u8],
    attrs: Vec<(&'a [u8], &'a [u8])>,
}

impl<'a> RssFeedParser<'a> {

    #[inline]
    fn parse_str(bytes: &'a [u8]) -> Result<Cow<'a, str>, Error> {
        Ok(std::str::from_utf8(bytes).map(Cow::Borrowed)?)
    }

    #[inline]
    fn resolve_namespace<'b>(&self, raw_name: &'b [u8]) -> (Namespace, &'b [u8]) {
        match raw_name.iter().position(|&b| b == b':') {
            Some(i) => {
                let (prefix, local) = (&raw_name[..i], &raw_name[i + 1..]);
                let ns = self.namespace_stack.iter().rev()
                    .find_map(|scope| scope.iter().find(|b| b.prefix == prefix).map(|b| b.ns))
                    .unwrap_or(Namespace::Custom);
                (ns, local)
            }
            None => (Namespace::Rss, raw_name),
        }
    }
    
    #[inline]
    fn parse_element_using_namespace(name: &'a [u8], ns: Namespace) -> RssElement {
        match ns {
            Namespace::Rss => RssElement::from(name),
            Namespace::Atom => RssElement::Atom(AtomElement::from(name)),
            Namespace::DublinCore => RssElement::DublinCore(DublinCoreLegacyNamespace::from(name)),
            //Namespace::Content => {},
            //Namespace::Media => {},
            //Namespace::Custom => {},
            _ => RssElement::Unknown,
        }
    }

    pub fn parse_rss_feed(&mut self) -> Result<(), Error> {

        let mut pending_start_tag: Option<PendingTag<'a>> = None;

        for token in self.tokenizer.by_ref() {
            
            match token {
                Ok(XmlToken::StartTag(name)) => {
                    pending_start_tag = Some(PendingTag { raw_name: name, attrs: Vec::new() });
                }

                Ok(XmlToken::Attribute { name, value }) => {
                    if let Some(pending) = pending_start_tag.as_mut() {
                        pending.attrs.push((name, value));
                    }
                }

                Ok(XmlToken::TagEnd { self_closing }) => {
                    let pending = pending_start_tag.take().expect("TagEnd without StartTag");
                   
                    // Parse namespaces. 
                    let mut namespaces = Vec::new();
                    for (name, value) in &pending.attrs {
                        if let Some(prefix) = name.strip_prefix(b"xmlns:") {
                            namespaces.push(NamespaceBinding { prefix, ns: Namespace::from_uri(value) });
                        }
                    }
                    self.namespace_stack.push(namespaces);

                    // And now process the raw_name.
                    let (prefix, local) = match pending.raw_name.iter().position(|&b| b == b':') {
                        Some(i) => (Some(&pending.raw_name[..i]), &pending.raw_name[i+1..]),
                        None => (None, pending.raw_name),
                    };

                    let (namespace, tag_name) = match prefix {
                        Some(prefix) => {
                            (self.namespace_stack
                                .iter()
                                .rev()
                                .find_map(|scope| scope.iter().find(|p| p.prefix == prefix).map(|ns| ns.ns))
                                .unwrap_or(Namespace::Custom),
                                local
                            )
                        }
                        None => (Namespace::Rss, pending.raw_name),
                    };
                    let element = Self::parse_element_using_namespace(tag_name, namespace);

                    self.open_element_stack.push((element, pending.raw_name));

                    // init elements with attributes.
                    match element {
                        RssElement::Item => {
                            self.items.push(RssItem::default());
                        }
                        RssElement::Image => self.channel.image = Some(RssImageElement::default()),
                        RssElement::TextInput => self.channel.text_input = Some(RssTextInputElement::default()),
                        RssElement::Category => {
                            if let Some(item) = self.items.last_mut() {
                                item.category = Some(RssItemCategory::default())
                            }
                        },
                        RssElement::Enclosure => {
                            if let Some(item) = self.items.last_mut() {
                                item.enclosure = Some(RssItemEnclosure::default())
                            }
                        },
                        RssElement::Guid => {
                            if let Some(item) = self.items.last_mut() {
                                item.guid = Some(RssItemGuid::default())
                            }
                        },
                        RssElement::Source => {
                            if let Some(item) = self.items.last_mut() {
                                item.source = Some(RssItemSource::default())
                            }
                        },
                        _ => {}
                    }
                    
                    // Assign attributes.
                    for (attr_name, value) in &pending.attrs {
                        if attr_name.starts_with(b"xmlns:") {
                            continue;
                        }

                    }

                    if self_closing {
                        self.open_element_stack.pop();
                        self.namespace_stack.pop();
                    }
                },
                
                Ok(XmlToken::EndTag(name)) => {
                    if let Some((_, previous_bytes)) = self.open_element_stack.last() {

                        if &name == previous_bytes {
                            self.open_element_stack.pop();
                            self.namespace_stack.pop();
                            continue;
                        }
                        return Err(Error::ElementMismatch { expected: String::from_utf8_lossy(previous_bytes).to_string(), found: String::from_utf8_lossy(name).to_string()});
                    }
                    unreachable!("End tag encountered with empty stack")
                    
                },

                Ok(XmlToken::Text(data)) | Ok(XmlToken::CharacterData(data)) => {

                    let parsed_data = RssFeedParser::parse_str(data)?;

                    let parent = self.open_element_stack
                        .iter()
                        .rev()
                        .nth(1)
                        .map(|(a, _)|a);
                    
                    // If items are empty, mutate channel.
                    if self.items.is_empty() {

                        let last_element = self.open_element_stack.last().map(|(a, _)|a);

                        match (last_element, parent) {

                            (Some(RssElement::Title), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.title = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Title), Some(RssElement::TextInput)) => {
                                if let Some(text_input) = self.channel.text_input.as_mut() {
                                    text_input.title = Some(parsed_data);
                                }
                            }
                            
                            (Some(RssElement::Title), _) => self.channel.title = Some(parsed_data),

                            (Some(RssElement::Link), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.link = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Link), Some(RssElement::TextInput)) => {
                                if let Some(text_input) = self.channel.text_input.as_mut() {
                                    text_input.link = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Link), _) => self.channel.link = Some(parsed_data),

                            (Some(RssElement::Description), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.description = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Description), Some(RssElement::TextInput)) => {
                                if let Some(text_input) = self.channel.text_input.as_mut() {
                                    text_input.description = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Description), _) => self.channel.description = Some(parsed_data),
                            (Some(RssElement::Language), _) => self.channel.language = Some(parsed_data),
                            (Some(RssElement::Copyright), _) => self.channel.copyright = Some(parsed_data),
                            (Some(RssElement::ManagingEditor), _) => self.channel.managing_editor = Some(parsed_data),
                            (Some(RssElement::WebMaster), _) => self.channel.web_master = Some(parsed_data),
                            (Some(RssElement::PubDate), _) => self.channel.pub_date = Some(parsed_data),
                            (Some(RssElement::LastBuildDate), _) => self.channel.last_build_date = Some(parsed_data),
                            (Some(RssElement::Category), _) => self.channel.category = Some(parsed_data),
                            (Some(RssElement::Generator), _) => self.channel.generator = Some(parsed_data),
                            (Some(RssElement::Docs), _) => self.channel.docs = Some(parsed_data),
                            (Some(RssElement::Cloud), _) => self.channel.cloud = Some(parsed_data),
                            (Some(RssElement::Ttl), _) => self.channel.ttl = Some(parsed_data),
                            //(Some(RssElement::Image), _) => {},
                            (Some(RssElement::Rating), _) => self.channel.rating = Some(parsed_data),
                            //(Some(RssElement::TextInput), _) => {},
                            (Some(RssElement::SkipHours), _) => self.channel.skip_hours = Some(parsed_data),
                            (Some(RssElement::SkipDays), _) => self.channel.skip_days = Some(parsed_data),

                            (Some(RssElement::Url), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.url = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Width), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.width = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Height), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.height = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Name), Some(RssElement::TextInput)) => {
                                if let Some(text_input) = self.channel.text_input.as_mut() {
                                    text_input.name = Some(parsed_data);
                                }
                            }

                            //_ => return Err(Error::UnexpectedRssChannelElement(parsed_data.to_string())),
                            _ => {}
                        }

                    } else {
                        if let Some(current_item) = self.items.last_mut() {
                            
                            let last_element = self.open_element_stack.last().map(|(a, _)|a);
                            
                            match (last_element, parent) {
                                (Some(RssElement::Title), _) => current_item.title = Some(parsed_data),
                                (Some(RssElement::Link), _) => current_item.link = Some(parsed_data),
                                (Some(RssElement::Description), _) => current_item.description = Some(parsed_data),
                                (Some(RssElement::Author), _) | (Some(RssElement::DublinCore(DublinCoreLegacyNamespace::Creator)), _) => current_item.author = Some(parsed_data),
                                (Some(RssElement::Category), _) => {
                                    if let Some(category) = current_item.category.as_mut() {
                                        category.value = Some(parsed_data)
                                    }
                                },
                                (Some(RssElement::Comments), _) => current_item.comments = Some(parsed_data),
                                //(Some(RssElement::Enclosure), _) => current_item.enclosure= Some(parsed_data),
                                (Some(RssElement::Guid), _) => {
                                    if let Some(guid) = current_item.guid.as_mut() {
                                        guid.value = Some(parsed_data)
                                    }
                                },
                                (Some(RssElement::PubDate), _) => current_item.pub_date= Some(parsed_data),
                                (Some(RssElement::Source), _) => {
                                    if let Some(source) = current_item.source.as_mut() {
                                        source.value = Some(parsed_data)
                                    }
                                }
                                _ => {},
                            }
                        }
                    }

                },


                

                _ => {}

                


            }
        }

        Ok(())
    }
    
    /*
    pub fn parse_rss_feed_from_tokenizer(&mut self) -> Result<(), Error> {
        
        for token in self.tokenizer.by_ref() {

            match token {
                Ok(XmlToken::Declaration(_)) => {},
                Ok(XmlToken::DeclarationTagEnd) => {},

                //Ok(XmlToken::StartTag(b"rss")) => {},

                //Ok(XmlToken::StartTag(b"channel")) => {},

                Ok(XmlToken::StartTag(b"item")) => {
                    self.items.push(RssItem::default());
                    self.open_elements.push(RssElement::Item);
                },

                Ok(XmlToken::StartTag(name)) => {
                    let element = RssElement::from(name);
                    
                    match element {
                        RssElement::Image => self.channel.image = Some(RssImageElement::default()),
                        RssElement::TextInput => self.channel.text_input = Some(RssTextInputElement::default()),
                        RssElement::Category => {
                            if let Some(item) = self.items.last_mut() {
                                item.category = Some(RssItemCategory::default())
                            }
                        },
                        RssElement::Enclosure => {
                            if let Some(item) = self.items.last_mut() {
                                item.enclosure = Some(RssItemEnclosure::default())
                            }
                        },
                        RssElement::Guid => {
                            if let Some(item) = self.items.last_mut() {
                                item.guid = Some(RssItemGuid::default())
                            }
                        },
                        RssElement::Source => {
                            if let Some(item) = self.items.last_mut() {
                                item.source = Some(RssItemSource::default())
                            }
                        },
                        _ => {}
                    }

                    self.open_elements.push(element);
                },
                
                Ok(XmlToken::TagEnd { self_closing }) if self_closing => {
                    self.open_elements.pop();
                },

                Ok(XmlToken::EndTag(name)) => {
                    if let Some(previous_element) = self.open_elements.last() {
                        let current_element = RssElement::from(name);
                        if &current_element == previous_element {
                            self.open_elements.pop();
                            continue;
                        }
                        return Err(Error::ElementMismatch { expected: *previous_element, found: current_element });
                    }
                    
                }

                Ok(XmlToken::Text(data)) | Ok(XmlToken::CharacterData(data)) => {

                    let parsed_data = RssFeedParser::parse_str(data)?;

                    let parent = self.open_elements.iter().rev().nth(1);
                    
                    // If items are empty, mutate channel.
                    if self.items.is_empty() {

                        match (self.open_elements.last(), parent) {

                            (Some(RssElement::Title), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.title = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Title), Some(RssElement::TextInput)) => {
                                if let Some(text_input) = self.channel.text_input.as_mut() {
                                    text_input.title = Some(parsed_data);
                                }
                            }
                            
                            (Some(RssElement::Title), _) => self.channel.title = Some(parsed_data),

                            (Some(RssElement::Link), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.link = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Link), Some(RssElement::TextInput)) => {
                                if let Some(text_input) = self.channel.text_input.as_mut() {
                                    text_input.link = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Link), _) => self.channel.link = Some(parsed_data),

                            (Some(RssElement::Description), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.description = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Description), Some(RssElement::TextInput)) => {
                                if let Some(text_input) = self.channel.text_input.as_mut() {
                                    text_input.description = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Description), _) => self.channel.description = Some(parsed_data),
                            (Some(RssElement::Language), _) => self.channel.language = Some(parsed_data),
                            (Some(RssElement::Copyright), _) => self.channel.copyright = Some(parsed_data),
                            (Some(RssElement::ManagingEditor), _) => self.channel.managing_editor = Some(parsed_data),
                            (Some(RssElement::WebMaster), _) => self.channel.web_master = Some(parsed_data),
                            (Some(RssElement::PubDate), _) => self.channel.pub_date = Some(parsed_data),
                            (Some(RssElement::LastBuildDate), _) => self.channel.last_build_date = Some(parsed_data),
                            (Some(RssElement::Category), _) => self.channel.category = Some(parsed_data),
                            (Some(RssElement::Generator), _) => self.channel.generator = Some(parsed_data),
                            (Some(RssElement::Docs), _) => self.channel.docs = Some(parsed_data),
                            (Some(RssElement::Cloud), _) => self.channel.cloud = Some(parsed_data),
                            (Some(RssElement::Ttl), _) => self.channel.ttl = Some(parsed_data),
                            //(Some(RssElement::Image), _) => {},
                            (Some(RssElement::Rating), _) => self.channel.rating = Some(parsed_data),
                            //(Some(RssElement::TextInput), _) => {},
                            (Some(RssElement::SkipHours), _) => self.channel.skip_hours = Some(parsed_data),
                            (Some(RssElement::SkipDays), _) => self.channel.skip_days = Some(parsed_data),

                            (Some(RssElement::Url), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.url = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Width), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.width = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Height), Some(RssElement::Image)) => {
                                if let Some(image) = self.channel.image.as_mut() {
                                    image.height = Some(parsed_data);
                                }
                            }

                            (Some(RssElement::Name), Some(RssElement::TextInput)) => {
                                if let Some(text_input) = self.channel.text_input.as_mut() {
                                    text_input.name = Some(parsed_data);
                                }
                            }

                            //_ => return Err(Error::UnexpectedRssChannelElement(parsed_data.to_string())),
                            _ => {}
                        }

                    } else {
                        if let Some(current_item) = self.items.last_mut() {
                            match (self.open_elements.last(), parent) {
                                (Some(RssElement::Title), _) => current_item.title = Some(parsed_data),
                                (Some(RssElement::Link), _) => current_item.link = Some(parsed_data),
                                (Some(RssElement::Description), _) => current_item.description = Some(parsed_data),
                                (Some(RssElement::Author), _) | (Some(RssElement::DublinCore(DublinCoreLegacyNamespace::Creator)), _) => current_item.author = Some(parsed_data),
                                (Some(RssElement::Category), _) => {
                                    if let Some(category) = current_item.category.as_mut() {
                                        category.value = Some(parsed_data)
                                    }
                                },
                                (Some(RssElement::Comments), _) => current_item.comments = Some(parsed_data),
                                //(Some(RssElement::Enclosure), _) => current_item.enclosure= Some(parsed_data),
                                (Some(RssElement::Guid), _) => {
                                    if let Some(guid) = current_item.guid.as_mut() {
                                        guid.value = Some(parsed_data)
                                    }
                                },
                                (Some(RssElement::PubDate), _) => current_item.pub_date= Some(parsed_data),
                                (Some(RssElement::Source), _) => {
                                    if let Some(source) = current_item.source.as_mut() {
                                        source.value = Some(parsed_data)
                                    }
                                }
                                _ => {},
                            }
                        }
                    }

                },

                Ok(XmlToken::Attribute { name, value }) => {
                    
                    let parsed_value = RssFeedParser::parse_str(value)?;

                    if let Some(current_item) = self.items.last_mut() {
                        match (self.open_elements.last(), name) {
                            

                            

                            (Some(RssElement::Source), b"url") => {
                                if let Some(source) = current_item.source.as_mut() {
                                    source.url = Some(parsed_value)
                                }
                            }

                            (Some(RssElement::Enclosure), b"url") => {
                                if let Some(enclosure) = current_item.enclosure.as_mut() {
                                    enclosure.url = Some(parsed_value)
                                }
                            }

                            (Some(RssElement::Enclosure), b"length") => {
                                if let Some(enclosure) = current_item.enclosure.as_mut() {
                                    enclosure.length = Some(parsed_value)
                                }
                            }

                            (Some(RssElement::Enclosure), b"type") => {
                                if let Some(enclosure) = current_item.enclosure.as_mut() {
                                    enclosure.enclosure_type = Some(parsed_value)
                                }
                            }

                            (Some(RssElement::Category), b"domain") => {
                                if let Some(category) = current_item.category.as_mut() {
                                    category.domain = Some(parsed_value)
                                }
                            }

                            (Some(RssElement::Guid), b"isPermaLink") => {
                                if let Some(guid) = current_item.guid.as_mut() {
                                    guid.is_perma_link = Some(parsed_value)
                                }
                            }


                            _ => {}
                        }
                    } else {                     
                                                
                        match (self.open_elements.last(), name) {
                            // Global namespaces
                            (Some(RssElement::Rss), _) if name.starts_with(b"xmlns:") => {
                                if let Some(prefix) = name.strip_prefix(b"xmlns:") {
                                    let ns = Namespace::from_uri(value);
                                    let extension = NamespaceBinding { prefix, ns };
                                    self.extensions.push(extension);
                                }
                            }
                            _ => {},
                        }
                    }

                },

                //_ => unimplemented!("parse_rss_feed_from_tokenizer")
                _ => {}

            }

       } 

        Ok(())
    }*/
}
