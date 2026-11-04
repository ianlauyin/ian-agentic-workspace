use crate::page::*;

#[test]
fn check_name_accepts_letters_digits_and_dash() {
    assert!(check_name("todo-items-2").is_ok());
}

#[test]
fn check_name_rejects_other_names() {
    for name in ["", "todo_items", "todo items", "../db", "todö"] {
        assert!(check_name(name).is_err(), "{name}");
    }
}
