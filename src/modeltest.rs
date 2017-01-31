// Tests for Model

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
    let text = String::from("Start\n\nEnd\n");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    test_model.add_block(String::from("This is a block"), 2, 1);

    // These should do nothing
    test_model.add_block(String::from("This is past the line"), 2, 25);
    test_model.add_block(String::from("Past EOF"), 4, 1);

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
    test_model.delete_char(1, 1); // this should do nothing
    test_model.delete_char(1, 11);

    assert_eq!("Text test\n", test_model.get_text());
}

#[test]
fn delete_block() {
    // TODO: protect out of bounds
    let text = String::from("Text.._block_ test\t\t_block");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    // TODO: it would make more sense for this to be (1, 5, 9)
    test_model.delete_block(1, 14, 9);
    assert_eq!("Text test\t\t_block\n", test_model.get_text());
    test_model.delete_block(1, 18, 8);
    assert_eq!("Text test\n", test_model.get_text());

    // These should do nothing
    test_model.delete_block(1, 1, 0);
    test_model.delete_block(3, 2, 1);

    assert_eq!("Text test\n", test_model.get_text());
}

#[test]
fn delete_line() {
    let text = String::from("First\nSecond\nThird\n");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    test_model.delete_line(2);

    // This should do nothing
    test_model.delete_line(4);

    assert_eq!("First\nThird\n", test_model.get_text());
}

#[test]
fn get_char() {
    let text = String::from("First\nSecond\n\t\tThird\n");
    let mut test_model = model::Model::new(text.as_str(), "/home/test/file");

    assert_eq!('F', test_model.get_char(1, 1));
    assert_eq!('t', test_model.get_char(1, 5));
    assert_eq!('S', test_model.get_char(2, 1));

    assert_eq!('\t', test_model.get_char(3, 1));
    assert_eq!('T', test_model.get_char(3, 3));

    assert_eq!('_', test_model.get_char(1, 10));
}

#[test]
fn get_line() {
    let text = String::from("First\nSecond\n\t\tThird\n");
    let test_model = model::Model::new(text.as_str(), "/home/test/file");

    assert_eq!("First", test_model.get_line(1));
    assert_eq!("Second", test_model.get_line(2));
    assert_eq!("\t\tThird", test_model.get_line(3));
    assert_eq!("", test_model.get_line(4));
}

#[test]
fn get_line_len() {
    let text = String::from("1234\n12345\n12\n");
    let test_model = model::Model::new(text.as_str(), "/home/test/file");

    assert_eq!(4, test_model.get_line_len(1));
    assert_eq!(5, test_model.get_line_len(2));
    assert_eq!(2, test_model.get_line_len(3));
    assert_eq!(0, test_model.get_line_len(4));
    assert_eq!(0, test_model.get_line_len(5));
}
