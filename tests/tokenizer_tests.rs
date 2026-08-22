/*
use serde::Deserialize;
use serde_json::{Map, Value, json};

use sift::html::tokens::HtmlToken;

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

        HtmlToken::Character(data) => Some(json!(["Character", data])),

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



#[test]
fn run_tokenizer_tests() {
    let test_dir = std::path::Path::new("tests/html5lib-tests/tokenizer");

    let entries = std::fs::read_dir(test_dir)
        .unwrap_or_else(|e| panic!("Failed to read test directory {:?}: {}", test_dir, e));

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();


        if path != std::path::Path::new("tests/html5lib-tests/tokenizer/contentModelFlags.test") {
            continue;
        }


        // Only process .test files
        if path.extension().and_then(|s| s.to_str()) != Some("test") {
            continue;
        }

        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read test file {:?}: {}", path, e));

        let suite: TestSuite = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse JSON in {:?}: {}", path, e));

        if let Some(tests) = suite.tests {
            for test in tests {
                let input = if test.double_escaped == Some(true) {
                    unescape(&test.input)
                } else {
                    test.input.clone()
                };

                let mut tokenizer = sift::html::tokens::HtmlTokenizer::new(&input);
                let mut raw_tokens = Vec::new(); 
                while let Some(token) = tokenizer.next_token() {
                    if token == HtmlToken::EndOfFile {
                        break;
                    }
                    raw_tokens.push(token);
                }

                let actual_output = format_tokens_for_test(&raw_tokens);

                assert_eq!(
                    actual_output, 
                    test.output,
                    "Failed test '{}' in file {:?}", 
                    test.description, 
                    path.file_name().unwrap()
                );
            }
        }
    }
}

/// Unescapes \uXXXX unicode sequences in doubleEscaped test strings using serde_json.
fn unescape(s: &str) -> String {
    serde_json::from_str(&format!("\"{}\"", s)).unwrap_or_else(|_| s.to_string())
}*/
