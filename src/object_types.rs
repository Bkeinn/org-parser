use std::u32::MAX;

use regex::Regex;

use crate::{builder, parser::count_initial_repeats, structs::{self, Priority, TodoStates}, time_management::{self, InfoType}};

/// The Object type specifies all types a Line in Orgmode could have.
/// If you would like to add a new type, this is the first place it should be added.
/// All possible types then have their attributes listed here.
/// If an attribute is Optional, it should be raped in an Option<>
/// If the attribute is mandetory, like the text in a heading(WHich could be just en empty string) it should be included in the dedicated build funktion
/// ## Value
/// Every Object has a value to compare if they are children of each other or if a new Object should start
/// EmptyLine: Has the lowest value an is therefore allways under the context oft the last Object
/// Text: Has the second lowest value an is therefor only above the Empty Line
/// ListElements: List are able to embed list in themselfes, but only up to 100, so list values range from 500-599 inclusive
/// Headings: are of high priority but are only allowed to recurse until 100, so heading values rango from 100-199 inclusive
#[derive(Debug)]
pub enum ObjectTypes {
    Heading {
        text: String,
        todo: Option<TodoStates>,
        /// The difference between schedule and in_line_scedule is, that the schedule is defined in the line underneath, while the in_line in the same line
        /// As there is currently no nead for the out of line schedule to move the information into the header. This is not supported
        deadline: Option<time_management::ParsedDateTime>,
        scheduled: Option<time_management::ParsedDateTime>,
        in_line_scedule: Option<time_management::ParsedDateTime>,
        value: u32,
        priority: Option<Priority>,
    },
    Text {
        text: String,
    },
    ListElement {
        text: String,
        todo: Option<TodoStates>,
        checkbox: Option<bool>,
        value: u32,
    },
    EmptyLine,
    INFO {
        info: Vec<InfoType>,
        text: String,
    },
    File {
        context: structs::FileContext,
    }
}

impl ObjectTypes {
    /// Values are ordered from 0-u32MAX where 0 is the highes values(live with it)
    /// This is used by the tree build funktion to order the Objects in parents and children
    pub fn value(&self) -> u32 {
        return match self {
            ObjectTypes::EmptyLine => MAX,
            ObjectTypes::Text { text } => MAX - 1,
            ObjectTypes::INFO { info, text } => MAX - 1,
            ObjectTypes::ListElement {
                text,
                todo,
                checkbox,
                value,
            } => *value + 500,
            ObjectTypes::Heading {
                text,
                todo,
                deadline,
                scheduled,
                value,
                priority,
                in_line_scedule,
            } => *value + 100,
            ObjectTypes::File { context } => 0,
            _ => MAX,
        };
    }
    /// The build funktion is used to create a String version of a specific object from their rust, datetype implementation
    /// I think it's called deserialisation
    pub fn build(&self) -> String {
        match self {
            ObjectTypes::EmptyLine => "".to_string(),
            ObjectTypes::Text { text } => text.to_owned(),
            ObjectTypes::INFO { info, text } => {
                let vector: Vec<String> = info
                    .iter()
                    .map(|inf| <InfoType as Clone>::clone(&inf).build())
                    .collect();
                return vector.join(" ");
            }
            ObjectTypes::ListElement {
                text,
                todo,
                checkbox,
                value,
            } => {
                return format!(
                    "{} {}{}{}",
                    builder::build_value(*value, ' ', '-'),
                    match checkbox {
                        Some(checkbox) => match *checkbox {
                            true => "[X] ",
                            false => "[ ] ",
                        },
                        None => "",
                    },
                    match todo {
                        Some(todo) => todo.build() + " ",
                        None => "".to_owned(),
                    },
                    text
                )
            }
            ObjectTypes::Heading {
                text,
                todo,
                deadline,
                scheduled,
                value,
                priority,
                in_line_scedule,
            } => {
                return format!(
                    "{} {}{}{}{}",
                    builder::build_value(*value, '*', '*'),
                    match todo {
                        Some(todo) => todo.build() + " ",
                        None => "".to_string(),
                    },
                    match priority {
                        Some(priority) => priority.build() + " ",
                        None => "".to_string(),
                    },
                    text,
                    match in_line_scedule {
                        Some(time) => time.build(),
                        None => "".to_string(),
                    }
                );
            }
            ObjectTypes::File { context } => format!("File context: {}", context.build()),
        }
    }
    /// Takes in a String which, should be a full line of org, and creates an ObjectType of Type Header
    /// It's is untested what happens when you give it a string that is not a header. So only give it allready identified strings
    pub fn new_heading(input: &str) -> ObjectTypes {
        return ObjectTypes::Heading {
            text: ObjectTypes::head_cleanup(input),
            todo: TodoStates::get("*", input),
            deadline: None,
            scheduled: None,
            in_line_scedule: ObjectTypes::inline_schedule(input),
            value: count_initial_repeats(input),
            priority: Priority::get(input),
        };
    }

    fn head_cleanup(input: &str) -> String {
        let re = Regex::new(r"^(\**)( *)(TODO|NEXT|DONE|LOOP|)").unwrap();
        let re_time = Regex::new(r"<[^>]*>").unwrap();

        let input = input.replace("[#C]", "");
        let input = input.replace("[#B]", "");
        let input = input.replace("[#A]", ""); // Remove just the first

        return re_time
            .replace_all(&re.replace_all(&input, ""), "")
            .trim()
            .to_string();
    }

    fn inline_schedule(input: &str) -> Option<time_management::ParsedDateTime> {
        let re_time = Regex::new(r"<[^>]*>").unwrap();
        return match re_time.find(input) {
            Some(mat) => time_management::ParsedDateTime::parse(mat.as_str()),
            None => None,
        };
    }

    /// Creates an new List Element from a string, only strings that where allready identified as strings which represent an org LiestELement should be passed to this funktion.
    pub fn new_list_element(input: &str) -> ObjectTypes {
        return ObjectTypes::ListElement {
            text: ObjectTypes::list_cleanup(input),
            todo: TodoStates::get("-", input),
            checkbox: ObjectTypes::get_checkbox(input),
            value: count_initial_repeats(input),
        };
    }
    fn list_cleanup(input: &str) -> String {
        let re = Regex::new(r"^( *)-(( *)\[( |X|/)\]|)( *)").unwrap();
        return re.replace(input, "").trim().to_string();
    }
    fn get_checkbox(input: &str) -> Option<bool> {
        let re_true = Regex::new(r"^( *)-( *)\[X\]( *)").unwrap();
        let re_false = Regex::new(r"^( *)-( *)\[ \]( *)").unwrap();
        if re_true.is_match(input) {
            return Some(true);
        } else if re_false.is_match(input) {
            return Some(false);
        } else {
            return None;
        }
    }
    /// Creates a new Textblock from a given String
    pub fn new_text(input: String) -> ObjectTypes {
        return ObjectTypes::Text { text: input };
    }
    /// Creates a new Infotype from a current String, only pass allready as Info type identified Strings
    /// These include Lines as
    /// SCHEDULED | DEADLINE | CLOSED
    pub fn new_info(input: String) -> ObjectTypes {
        let mut info = ObjectTypes::INFO {
            info: Vec::new(),
            text: input.trim().to_string().clone(),
        };
        info.info_add_info(&input);
        return info;
    }

    fn info_add_info(&mut self, input: &str) {
        let re = Regex::new(r"(DEADLINE|SCHEDULED): <[^>]+>").unwrap();
        match self {
            ObjectTypes::INFO { info, text } => {
                for mat in re.find_iter(input) {
                    match time_management::InfoType::get(mat.as_str()) {
                        Some(value) => info.push(value),
                        None => (),
                    }
                }
            }
            _ => (),
        }
    }
    /// Creates a new Empty line
    pub fn new_empty() -> ObjectTypes {
        return ObjectTypes::EmptyLine;
    }
}
