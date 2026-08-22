
// https://html.spec.whatwg.org/#parse-errors
#[derive(Debug, PartialEq)]
pub enum Error {
    AbruptClosingOfEmptyComment,

    AbruptDoctypePublicIdentifier,

    AbruptDoctypeSystemIdentifier,

    AbsenceOfDigitsInNumericCharacterReference,

    CdataInHtmlContent,

    CharacterReferenceOutsideUnicodeRange,

    ControlCharacterInInputStream,

    ControlCharacterReference,

    DisallowedProcessingInstructionTarget,

    DuplicateAttribute,

    EndTagWithAttributes,

    EndTagWithTrailingSolidus,

    EofBeforeTagName,

    EofInCdata,

    EofInComment,

    EofInDoctype,

    EofInProcessingInstruction,

    EofInScriptHtmlCommentLikeText,

    EofInTag,

    IncorrectlyClosedComment,

    IncorrectlyOpenedComment,

    InvalidCharacterSequenceAfterDoctypeName,

    InvalidFirstCharacterOfProcessingInstructionTarget,

    InvalidFirstCharacterOfTagName,

    InvalidProcessingInstructionTarget,

    MissingAttributeValue,

    MissingDoctypeName,

    MissingDoctypePublicIdentifier,

    MissingDoctypeSystemIdentifier,

    MissingEndTagName,

    MissingQuoteBeforeDoctypePublicIdentifier,

    MissingQuoteBeforeDoctypeSystemIdentifier,

    MissingSemicolonAfterCharacterReference,

    MissingWhitespaceAfterDoctypePublicKeyword,

    MissingWhitespaceAfterDoctypeSystemKeyword,

    MissingWhitespaceBeforeDoctypeName,

    MissingWhitespaceBetweenAttributes,

    MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers,

    NestedComment,

    NoncharacterCharacterReference,

    NoncharacterInInputStream,

    NonVoidHtmlElementStartTagWithTrailingSolidus,

    NullCharacterReference,

    SurrogateCharacterReference,

    SurrogateInInputStream,

    UnexpectedCharacterAfterDoctypeSystemIdentifier,

    UnexpectedCharacterInAttributeName,

    UnexpectedCharacterInUnquotedAttributeValue,

    UnexpectedEqualsSignBeforeAttributeName,

    UnexpectedNullCharacter,

    UnexpectedSolidusInTag,

    UnknownNamedCharacterReference,
}
