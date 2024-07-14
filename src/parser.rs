use regex::Regex;

use crate::structs::{self};
use crate::object_types::ObjectTypes;
use crate::object::Object;


impl structs::Context {
    pub fn parse(&mut self, text: Vec<String>) {
        for line in text {
            self.lines.push((parse_line(line.clone()), line));
        }
    }
}

fn parse_line(text: String) -> ObjectTypes {
    let re_heading = Regex::new(r"^\s*\*").unwrap();
    let re_list = Regex::new(r"^\s*-").unwrap();
    let re_comment = Regex::new(r"^\s*#").unwrap();
    let re_info = Regex::new(r"^\s*(SCHEDULED|DEADLINE|CLOSED)").unwrap();
    let re_empty = Regex::new(r"^\s*$").unwrap();

    let mut object: ObjectTypes = ObjectTypes::EmptyLine;

    if re_heading.is_match(&text) {
        object = ObjectTypes::new_heading(&text);
    } else if re_list.is_match(&text) {
        object = ObjectTypes::new_list_element(&text);
    } else if re_comment.is_match(&text) {
        object = ObjectTypes::new_text(text);
    } else if re_info.is_match(&text) {
        object = ObjectTypes::new_info(text);
    } else if re_empty.is_match(&text) {
        object = ObjectTypes::new_empty();
    } else {
        object = ObjectTypes::new_text(text);
    }

    return object;
}

pub fn count_initial_repeats(input: &str) -> u32 {
    if input.is_empty() {
        return 0;
    }

    let chars: Vec<char> = input.chars().collect();
    let first_char = chars[0];
    let mut count = 0;

    for &c in &chars {
        if c == first_char {
            count += 1;
        } else {
            break;
        }
    }

    count
}
