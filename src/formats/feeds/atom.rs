// https://www.greenbuttonalliance.org/atom-elements

use std::borrow::Cow;

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AtomElement {
    Feed,
    Id,
    Link,
    Title,
    Published,
    Updated,
    Entry,
    Content,
    Unknown,
}

impl<'a> From<&'a [u8]> for AtomElement {

    fn from(value: &'a [u8]) -> Self {
        match value {
            b"feed" => Self::Feed,
            b"id" => Self::Id,
            b"link" => Self::Link,
            b"title" => Self::Title,
            b"published" => Self::Published,
            b"updated" => Self::Updated,
            b"entry" => Self::Entry,
            b"content" => Self::Content,
            _ => Self::Unknown,

        }
    }
}

#[derive(Debug, Default)]
pub struct AtomLink<'a> {
    rel: Option<Cow<'a, str>>,
    href: Option<Cow<'a, str>>,
    kind: Option<Cow<'a, str>>,
}
