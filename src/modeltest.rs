use super::model;

#[test]
fn new() {
    let text = String::from("This is a test");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    assert_eq!("This is a test", test_model.get_text());
}
