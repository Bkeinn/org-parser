use regex::Regex;

use crate::structs::{self, Context, Object, ObjectTypes};

impl structs::Object {
    /// It is important to parse the text without the object_type
    pub fn parse(context: structs::Context) -> structs::Object {
        let mut lowest_value = u32::MAX;
        let mut obj_context: structs::Context = structs::Context::new();
        let mut result_obj = structs::Object::empty();
        for (number, (obj, string)) in context.lines.into_iter().enumerate() {
            if number == 1 {
                lowest_value = obj.value();
                result_obj = structs::Object::new(obj);
            } else if obj.value() > lowest_value {
                obj_context.add_context_line((obj, string));
            } else {
                let temp_context = std::mem::take(&mut obj_context);
                result_obj.add_child(Object::parse(temp_context));
                lowest_value = obj.value();
            }
        }

        return result_obj;
    }
}

impl structs::File {
    pub fn parse(&mut self, text: Vec<String>) {
        for line in text {
            self.context.lines.push((parse_line(line.clone()), line));
        }
    }
}

fn parse_line(text: String) -> structs::ObjectTypes {
    let re_heading = Regex::new(r"^\s*\*").unwrap();
    let re_list = Regex::new(r"^\s*-").unwrap();
    let re_comment = Regex::new(r"^\s*#").unwrap();
    let re_info = Regex::new(r"^\s*(SCHEDULED|DEADLINE|CLOSED)").unwrap();
    let re_empty = Regex::new(r"^\s*$").unwrap();

    let mut object: structs::ObjectTypes = structs::ObjectTypes::EmptyLine;

    if re_heading.is_match(&text) {
        object = structs::ObjectTypes::new_heading(&text);
    } else if re_list.is_match(&text) {
        object = structs::ObjectTypes::new_list_element(&text);
    } else if re_comment.is_match(&text) {
        object = structs::ObjectTypes::new_text(text);
    } else if re_info.is_match(&text) {
        object = structs::ObjectTypes::new_info(text);
    } else if re_empty.is_match(&text) {
        object = structs::ObjectTypes::new_empty();
    } else {
        object = structs::ObjectTypes::new_text(text);
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
