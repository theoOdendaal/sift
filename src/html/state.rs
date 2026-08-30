#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenizationState {
    Data,

    RcData,

    RawText,

    ScriptData,

    PlainText,

    TagOpen,

    EndTagOpen,

    TagName,

    RcDataLessThanSign,

    RcDataEndTagOpen,

    RcDataEndTagName,

    RawTextLessThanSign,

    RawTextEndTagOpen,

    RawTextEndTagName,

    ScriptDataLessThanSign,

    ScriptDataEndTagOpen,

    ScriptDataEndTagName,

    ScriptDataEscapeStart,

    ScriptDataEscapeStartDash,

    ScriptDataEscaped,

    ScriptDataEscapedDash,

    ScriptDataEscapedDashDash,

    ScriptDataEscapedLessThanSign,

    ScriptDataEscapedEndTagOpen,

    ScriptDataEscapedEndTagName,

    ScriptDataDoubleEscapeStart,

    ScriptDataDoubleEscaped,

    ScriptDataDoubleEscapedDash,

    ScriptDataDoubleEscapedDashDash,

    ScriptDataDoubleEscapedLessThanSign,

    ScriptDataDoubleEscapeEnd,

    BeforeAttributeName,

    AttributeName,

    AfterAttributeName,

    BeforeAttributeValue,

    AttributeValueDoubleQuoted,

    AttributeValueSingleQuoted,

    AttributeValueUnquoted,

    AfterAttributeValueQuoted,

    SelfClosingStartTag,

    BogusComment,

    MarkupDeclarationOpen,

    CommentStart,

    CommentStartDash,

    Comment,

    CommentLessThanSign,

    CommentLessThanSignBang,

    CommentLessThanSignBangDash,

    CommentLessThanSignBangDashDash,

    CommentEndDash,

    CommentEnd,

    CommentEndBang,

    Doctype,

    BeforeDoctypeName,

    DoctypeName,

    AfterDoctypeName,

    AfterDoctypePublicKeyword,

    BeforeDoctypePublicIdentifier,

    DoctypePublicIdentifierDoubleQuoted,

    DoctypePublicIdentifierSingleQuoted,

    AfterDoctypePublicIdentifier,

    BetweenDoctypePublicAndSystemIdentifiers,

    AfterDoctypeSystemKeyword,

    BeforeDoctypeSystemIdentifier,

    DoctypeSystemIdentifierDoubleQuoted,

    DoctypeSystemIdentifierSingleQuoted,

    AfterDoctypeSystemIdentifier,

    BogusDoctype,

    CdataSection,

    CdataSectionBracket,

    CdataSectionEnd,

    ProcessingInstructionOpen,

    ProcessingInstructionTarget,

    AfterProcessingInstructionTarget,

    ProcessingInstructionData,

    ProcessingInstructionQuestionable,

    CharacterReference,

    NamedCharacterReference,

    AmbiguousAmpersand,

    NumericCharacterReference,

    HexadecimalCharacterReferenceStart,

    HexadecimalCharacterReference,

    DecimalCharacterReference,

    NumericCharacterReferenceEnd,
}
