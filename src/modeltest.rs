use super::model;

#[test]
fn get_text() {
    let text = String::from("This is a test\n");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    assert_eq!("This is a test\n", test_model.get_text());
}

#[test]
fn add_char_start() {
    let text = String::from("tart");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    test_model.add_char('s', 1, 1);

    assert_eq!("start\n", test_model.get_text());
}

#[test]
fn add_char_middle() {
    let text = String::from("Text to :: test with");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    test_model.add_char('x', 1, 10);

    assert_eq!("Text to :x: test with\n", test_model.get_text());
}

#[test]
fn add_char_end() {
    let text = String::from("Start\nSecond lin");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    test_model.add_char('e', 2, 11);

    assert_eq!("Start\nSecond line\n", test_model.get_text());
}
