use serde::Deserialize;
use serde_json::{Map, Value, json};

use sift::html::{
    state::TokenizationState,
    tokens::{HtmlToken, HtmlTokenizer},
};

#[derive(Deserialize, Debug)]
pub struct TestSuite {
    pub tests: Option<Vec<TokenizerTest>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenizerTest {
    pub description: String,
    pub input: String,
    pub output: Vec<Value>,
    pub initial_states: Option<Vec<String>>,
    pub double_escaped: Option<bool>,
    pub last_start_tag: Option<String>,
}

pub fn token_to_test_format(token: &HtmlToken) -> Option<Value> {
    match token {
        HtmlToken::StartTag(tag) => {
            let name = tag.name.as_deref().unwrap_or("");
            let mut attributes_map = Map::new();

            for attr in &tag.attributes {
                let key = attr.name.as_deref().unwrap_or("").to_string();
                let val = Value::String(attr.value.as_deref().unwrap_or("").to_string());
                // html5lib requires keeping ONLY the first occurrence of duplicate attributes
                attributes_map.entry(key).or_insert(val);
            }

            let attrs_value = Value::Object(attributes_map);

            if tag.self_closing_tag == Some(true) {
                Some(json!(["StartTag", name, attrs_value, true]))
            } else {
                Some(json!(["StartTag", name, attrs_value]))
            }
        }

        HtmlToken::EndTag(tag) => {
            let name = tag.name.as_deref().unwrap_or("");
            Some(json!(["EndTag", name]))
        }

        HtmlToken::Comment(data) => Some(json!(["Comment", data])),

        HtmlToken::Character(data) => {
            if !data.is_empty() {
                return Some(json!(["Character", data]));
            }
            None
        }

        HtmlToken::Doctype(doctype) => {
            let name = doctype.name.as_deref();
            let pub_id = doctype.public_identifier.as_deref();
            let sys_id = doctype.system_identifier.as_deref();

            let correctness = !doctype.force_quirks_flag;

            Some(json!(["DOCTYPE", name, pub_id, sys_id, correctness]))
        }

        HtmlToken::EndOfFile => None,
    }
}

pub fn format_tokens_for_test(tokens: &[HtmlToken]) -> Vec<Value> {
    let mut result: Vec<Value> = Vec::new();

    for token in tokens {
        let converted = match token_to_test_format(token) {
            Some(val) => val,
            None => continue,
        };

        if let Some(Value::Array(last)) = result.last_mut() {
            if last.get(0) == Some(&Value::String("Character".into())) {
                if let Value::Array(ref current) = converted {
                    if current.get(0) == Some(&Value::String("Character".into())) {
                        if let (Some(Value::String(prev_str)), Some(Value::String(curr_str))) =
                            (last.get_mut(1), current.get(1))
                        {
                            prev_str.push_str(curr_str);
                            continue;
                        }
                    }
                }
            }
        }

        result.push(converted);
    }

    result
}

fn parse_json_test_suite(file: &std::path::Path) -> TestSuite {
    let content =
        std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("Failed to read file: {}", e));

    let suite: TestSuite = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse JSON for file: {}", e));

    suite
}

fn parse_test_suite_state(state: &str) -> TokenizationState {
    match state {
        "PLAINTEXT state" => TokenizationState::PlainText,
        "RCDATA state" => TokenizationState::RcData,
        "RAWTEXT state" => TokenizationState::RawText,
        "Script data state" => TokenizationState::ScriptData,
        "Data state" => TokenizationState::Data,
        "CDATA section state" => TokenizationState::Data,
        _ => panic!("Unable to parse state: {}", state),
    }
}

fn generate_tokenizers<'a>(test: &'a TokenizerTest, input: &'a str) -> Vec<HtmlTokenizer<'a>> {
    let make_tokenizer = || {
        let mut tokenizer = HtmlTokenizer::new(input);
        if let Some(last_start_tag) = &test.last_start_tag {
            tokenizer.set_last_start_tag_name(last_start_tag);
        }
        tokenizer
    };

    let mut tokenizers = Vec::new();

    if let Some(initial_states) = &test.initial_states {
        for state_str in initial_states {
            let state = parse_test_suite_state(state_str);
            let mut tokenizer = make_tokenizer();
            tokenizer.set_state(state);
            tokenizers.push(tokenizer);
        }
    } else {
        tokenizers.push(make_tokenizer());
    }

    tokenizers
}

/// Unescapes \uXXXX unicode sequences in doubleEscaped test strings using serde_json.
fn unescape(s: &str) -> String {
    serde_json::from_str(&format!("\"{}\"", s)).unwrap_or_else(|_| s.to_string())
}

fn run_tokenizer_test(test_file: &std::path::Path, test: &TokenizerTest) {
    let input = if test.double_escaped == Some(true) {
        unescape(&test.input)
    } else {
        test.input.clone()
    };

    let tokenizers = generate_tokenizers(test, &input);

    for mut t in tokenizers {
        let mut raw_tokens = Vec::new();
        while let Some(token) = t.next_token() {
            if token == HtmlToken::EndOfFile {
                break;
            }
            raw_tokens.push(token);
        }
        let actual_output = format_tokens_for_test(&raw_tokens);

        // FIXME: I also need to compare errors.

        assert_eq!(
            test.output,
            actual_output,
            "Failed test '{}' in file {:?}",
            test.description,
            test_file.file_name().unwrap()
        );
    }
}

fn run_test_suite(file: &std::path::Path) {
    let suite = parse_json_test_suite(&file);

    if let Some(test_suite) = suite.tests {
        for test in test_suite {
            run_tokenizer_test(file, &test);
        }
    } else {
        panic!("Unable to load tests from: {:?}", file);
    }
}

#[test]
fn test_content_model_flags() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/contentModelFlags.test");
    run_test_suite(file);
}
/*
#[test]
fn test_domjs() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/domjs.test");
    run_test_suite(file);
}

#[test]
fn test_entities() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/entities.test");
    run_test_suite(file);
}
*/

#[test]
fn test_escape_flag() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/escapeFlag.test");
    run_test_suite(file);
}

/*

#[test]
fn test_named_entities() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/namedEntities.test");
    run_test_suite(file);
}


#[test]
fn test_numeric_entities() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/numericEntities.test");
    run_test_suite(file);
}

#[test]
fn test_pending_spec_changes() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/pendingSpecChanges.test");
    run_test_suite(file);
}
*/

#[test]
fn test_test1() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/test1.test");
    run_test_suite(file);
}
/*

#[test]
fn test_test2() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/test2.test");
    run_test_suite(file);
}

#[test]
fn test_test3() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/test3.test");
    run_test_suite(file);
}

#[test]
fn test_test4() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/test4.test");
    run_test_suite(file);
}
*/

#[test]
fn test_unicode_chars() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/unicodeChars.test");
    run_test_suite(file);
}
/*
#[test]
fn test_unicode_problematic() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/unicodeCharsProblematic.test");
    run_test_suite(file);
}

#[test]
fn test_xml_violation() {
    let file = std::path::Path::new("tests/html5lib-tests/tokenizer/xmlViolation.test");
    run_test_suite(file);
}
*/
