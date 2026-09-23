#[derive(Debug)]
pub struct DublinCoreParseError;

impl std::fmt::Display for DublinCoreParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Byte parse error")
    }
}

impl std::error::Error for DublinCoreParseError {}

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
}


impl<'a> TryFrom<&'a [u8]> for DublinCoreLegacyNamespace {
    type Error = DublinCoreParseError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value {
            b"dc:contributor" => Ok(Self::Contributor),
            b"dc:coverage" => Ok(Self::Coverage),
            b"dc:creator" => Ok(Self::Creator),
            b"dc:date" => Ok(Self::Date),
            b"dc:description" => Ok(Self::Description),
            b"dc:format" => Ok(Self::Format),
            b"dc:identifier" => Ok(Self::Identifier),
            b"dc:language" => Ok(Self::Language),
            b"dc:publisher" => Ok(Self::Publisher),
            b"dc:relation" => Ok(Self::Relation),
            b"dc:rights" => Ok(Self::Rights),
            b"dc:subject" => Ok(Self::Subject),
            b"dc:title" => Ok(Self::Title),
            b"dc:type" => Ok(Self::Type),
            _ => Err(DublinCoreParseError),

        }
    }
}
