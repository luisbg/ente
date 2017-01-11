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

#[test]
fn get_line_count() {
    let text = String::from("1\n2\n3\n");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    assert_eq!(3, test_model.get_line_count());

    test_model.add_char('4', 4, 1);
    test_model.add_char('\n', 4, 2);

    assert_eq!(4, test_model.get_line_count());
}

#[test]
fn add_block() {
    // TODO: Test block at line past end and test blocks past characters in line
    let text = String::from("Start\n\nEnd\n");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    test_model.add_block(String::from("This is a block"), 2, 1);
    assert_eq!("Start\nThis is a block\nEnd\n",
               test_model.get_text());
}

#[test]
fn delete_char() {
    // TODO: test out of bounds
    let text = String::from("Text _test_");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    // TODO: it would make more sense for this to be character 6
    test_model.delete_char(1, 7);
    test_model.delete_char(1, 1);  // this should do nothing
    test_model.delete_char(1, 11);

    assert_eq!("Text test\n", test_model.get_text());
}

#[test]
fn delete_block() {
    // TODO: protect out of bounds
    let text = String::from("Text.._block_ test");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    // TODO: it would make more sense for this to be (1, 5, 9)
    test_model.delete_block(1, 14, 9);

    // These should do nothing
    test_model.delete_block(1, 1, 0);
    test_model.delete_block(3, 2, 1);

    assert_eq!("Text test\n", test_model.get_text());
}
