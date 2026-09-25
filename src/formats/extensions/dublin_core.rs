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
            b"contributor" => Self::Contributor,
            b"coverage" => Self::Coverage,
            b"creator" => Self::Creator,
            b"date" => Self::Date,
            b"description" => Self::Description,
            b"format" => Self::Format,
            b"identifier" => Self::Identifier,
            b"language" => Self::Language,
            b"publisher" => Self::Publisher,
            b"relation" => Self::Relation,
            b"rights" => Self::Rights,
            b"subject" => Self::Subject,
            b"title" => Self::Title,
            b"type" => Self::Type,
            _ => Self::Unknown,

        }
    }
}
