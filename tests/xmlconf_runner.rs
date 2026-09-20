// FIXME: I will get back to this another day.

/*use std::{collections::HashMap, path::PathBuf};

use sift::xml::tokens::{XmlToken, XmlTokenizer};


const MANIFEST_DIRECTORY: &str = "tests/xmlconf/xmlconf.xml";

#[derive(Debug)]
struct TestCase {
    profile: String,
    base: String,
    path: String,
}

#[derive(Debug)]
struct Test {
    path: PathBuf,
    test_type: String,
    entities: String,
    id: String,
    uri: String,
    sections: String,
    description: String,
}

#[derive(Debug, PartialEq)]
enum State {
    Normal,
    TestCase,
}

#[test]
fn test_load_manifest() {
    let tests = load_tests_from_manifest().expect("Failed to parse manifest");

    for test in tests {
        if test.test_type == "valid" {
            println!("{:?}", test)
        }
    }
}


fn load_tests_from_manifest() -> Result<Vec<Test>, Box<dyn std::error::Error>> {
    let mut test_cases = Vec::new();
    let mut entity_declarations = HashMap::new();

    let content = std::fs::read(MANIFEST_DIRECTORY)?;
    let tokenizer = XmlTokenizer::from(content.as_slice());

    let mut state = State::Normal;
    let mut current_profile = String::new();
    let mut current_base = String::new();

    for t in tokenizer {
        match t {
            Ok(XmlToken::EntityDeclaration { name, identifier: _, literal }) => {
                let name = String::from_utf8_lossy(name).trim().to_string();
                let uri = String::from_utf8_lossy(literal).trim().strip_prefix("\"").and_then(|n| n.strip_suffix("\"")).unwrap().to_string();
                entity_declarations.insert(name, uri);
            },
            Ok(XmlToken::StartTag(name)) if name == b"TESTCASES" => {
                state = State::TestCase;
            },
            Ok(XmlToken::Attribute { name, value }) if state == State::TestCase && name == b"PROFILE" => {
                current_profile = String::from_utf8_lossy(value).trim().to_string();
            },
            Ok(XmlToken::Attribute { name, value }) if state == State::TestCase && name == b"xml:base" => {
                current_base = String::from_utf8_lossy(value).trim().to_string();
            },
            Ok(XmlToken::Text(text)) if state == State::TestCase => {
                
                for test_case in text.split(|&b| b == b'\n') {

                    let mut reference = String::from_utf8_lossy(test_case).trim().to_string();

                    if reference.is_empty() {
                        continue;
                    }

                    reference = reference.strip_prefix("&").expect("& expected").to_string();
                    reference = reference.strip_suffix(";").expect("; expected").to_string();

                    if let Some(uri) = entity_declarations.get(&reference) {
                        test_cases.push(TestCase {
                            profile: current_profile.clone(),
                            base: current_base.clone(),
                            path: uri.clone() 
                        });
                    }
                }
            },
            Ok(XmlToken::EndTag(name)) if name == b"TESTCASES" => {
                state = State::Normal;
            },
            _ => {}
        }
    }
    
    let mut test_type = String::new();
    let mut entities = String::new();
    let mut id = String::new();
    let mut uri = String::new();
    let mut sections = String::new();
    let mut description = String::new();

    let mut in_test = false;
    
    let mut tests = Vec::new();

    for test_case in &test_cases {
        
        let test_path = std::path::Path::new("tests/xmlconf").join(test_case.path.clone());
        let content = std::fs::read(&test_path)?;
        let tokenizer = XmlTokenizer::from(content.as_slice());

        for token in tokenizer {
            match token {
                Ok(XmlToken::StartTag(name)) if name == b"TEST" => {
                    in_test = true;
                },
                Ok(XmlToken::Attribute { name, value }) if in_test && name == b"TYPE" => {
                    test_type = String::from_utf8_lossy(value).to_string();
                },
                Ok(XmlToken::Attribute { name, value }) if in_test && name == b"ENTITIES" => {
                    entities = String::from_utf8_lossy(value).to_string();
                },
                Ok(XmlToken::Attribute { name, value }) if in_test && name == b"ID" => {
                    id = String::from_utf8_lossy(value).to_string();
                }
                Ok(XmlToken::Attribute { name, value }) if in_test && name == b"URI" => {
                    uri = String::from_utf8_lossy(value).to_string();
                }
                Ok(XmlToken::Attribute { name, value }) if in_test && name == b"SECTIONS" => {
                    sections = String::from_utf8_lossy(value).to_string();
                }
                Ok(XmlToken::Text(text)) if in_test => {
                    description = String::from_utf8_lossy(text).trim().to_string();
                }
                Ok(XmlToken::EndTag(name)) if name == b"TEST" => {
                    in_test = false;
                    tests.push(Test {
                            path: test_path.clone(),
                            test_type: test_type.clone(),
                            entities: entities.clone(),
                            id: id.clone(),
                            uri: uri.clone(),
                            sections: sections.clone(),
                            description: description.clone()
                    });
                }
                _ => {},
            }
        }
    }
    Ok(tests)
}*/
