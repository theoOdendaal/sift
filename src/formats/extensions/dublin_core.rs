#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DublinCoreLegacyNamespace {
    Contributor,
    Coverage,
    Creator,
    Date,
    Description,
    Format,
    Identifier,
    Language,
    Publisher,
    Relation,
    Rights,
    Subject,
    Title,
    Type,
    Unknown,
}


impl<'a> From<&'a [u8]> for DublinCoreLegacyNamespace {

    fn from(value: &'a [u8]) -> Self {
        match value {
            b"dc:contributor" => Self::Contributor,
            b"dc:coverage" => Self::Coverage,
            b"dc:creator" => Self::Creator,
            b"dc:date" => Self::Date,
            b"dc:description" => Self::Description,
            b"dc:format" => Self::Format,
            b"dc:identifier" => Self::Identifier,
            b"dc:language" => Self::Language,
            b"dc:publisher" => Self::Publisher,
            b"dc:relation" => Self::Relation,
            b"dc:rights" => Self::Rights,
            b"dc:subject" => Self::Subject,
            b"dc:title" => Self::Title,
            b"dc:type" => Self::Type,
            _ => Self::Unknown,

        }
    }
}
