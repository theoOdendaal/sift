
#[test]
fn run_w3c_xml_conformance_test_suites() {

    let manifest_path = std::path::Path::new("tests/xmlconf/xmlconf.xml");

    if !manifest_path.exists() {
        panic!("w3c XML Conformance Test Suites not found, please download from: https://www.w3.org/XML/Test/")
    }
    
    let manifest_content = std::fs::read_to_string(manifest_path).expect("Unable to read w3c XML Conformance Test Suites manifest");
    let tokenizer = sift::xml::tokens::XmlTokenizer::new(&manifest_content);

    for token in tokenizer {
        println!("{:?}", token);
        if let Err(_) = token {
            break;
        }
    }

}
