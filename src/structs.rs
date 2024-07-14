use crate::{builder, parser::count_initial_repeats};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use std::{default, u32::MAX};
use clap::ValueEnum;


/// Constructor funktions will allways require the whole Line, not just snipets
use regex::{self, Regex, Replacer};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FileContext {
    Todo,
}

impl FileContext {
    pub fn build(&self) -> &str {
        match self {
            FileContext::Todo => "Todo file",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Repeat {
    Dayly,
    Weekly,
    Monthly,
    Yearly,
}

impl Repeat {
    fn build(&self) -> char {
        match self {
            Repeat::Dayly => 'd',
            Repeat::Weekly => 'w',
            Repeat::Monthly => 'm',
            Repeat::Yearly => 'y',
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedDateTime {
    pub date: NaiveDate,
    pub day: String,
    pub repeat: Option<Repeat>,
}
/// Returns true when the date was changed, false if it was not changed
impl ParsedDateTime {
    fn update(&mut self) -> bool {
        let tody = chrono::Utc::now().date_naive();
        if self.date < tody {
            if let Some(repeater) = self.repeat {
                match repeater {
                    Repeat::Dayly => self.date = self.date + Duration::days(1),
                    Repeat::Weekly => self.date = self.date + Duration::weeks(1),
                    Repeat::Monthly => self.date = self.date + Duration::days(30),// Months are now 30 days, fact, pull request if you have a better idea
                    Repeat::Yearly => self.date = self.date + Duration::days(365),// Fuck Schaltjahre
                }
                return true;
            }
        }
        return false;
    }
    fn build(&self) -> String {
        let repeat = match &self.repeat {
            Some(rep) => format!(" .+l{}", rep.build()),
            None => "".to_owned(),
        };
        return format!("<{} {}{repeat}>", self.date.to_string(), self.day);
    }
    pub fn parse(input: &str) -> Option<Self> {
        let re = Regex::new(r"<(\d{4}-\d{2}-\d{2}) (\w{3})( \.\+l(d|w|m|y)|)>").unwrap();

        if let Some(caps) = re.captures(input) {
            let date_str = caps.get(1)?.as_str();
            let day_str = caps.get(2)?.as_str().to_string();
            let repeat = match caps.get(3)?.as_str().chars().last() {
                Some(character) => match character {
                    'd' => Some(Repeat::Dayly),
                    'w' => Some(Repeat::Weekly),
                    'm' => Some(Repeat::Monthly),
                    'y' => Some(Repeat::Yearly),
                    _ => None,
                },
                None => None,
            };

            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;

            Some(ParsedDateTime {
                date,
                repeat,
                day: day_str,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum InfoType {
    SCHEDULED { date: ParsedDateTime },
    DEADLINE { date: ParsedDateTime },
}

impl InfoType {
    /// Does not want multible
    fn get(input: &str) -> Option<InfoType> {
        let re_dead = Regex::new("DEADLINE:").unwrap();
        let re_sche = Regex::new("SCHEDULED:").unwrap();
        if re_dead.is_match(input) {
            return Some(InfoType::DEADLINE {
                date: match ParsedDateTime::parse(input) {
                    Some(datetime) => datetime,
                    None => return None,
                },
            });
        } else if re_sche.is_match(input) {
            return Some(InfoType::SCHEDULED {
                date: match ParsedDateTime::parse(input) {
                    Some(datetime) => datetime,
                    None => return None,
                },
            });
        } else {
            return None;
        }
    }
    fn build(self) -> String {
        match self {
            InfoType::DEADLINE { date } => format!("DEADLINE: {}", date.build()),
            InfoType::SCHEDULED { date } => format!("SCHEDULED: {}", date.build()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TodoStates {
    TODO,
    DONE,
    LOOP,
    NEXT,
}

impl TodoStates {
    fn get(base: &str, input: &str) -> Option<TodoStates> {
        let re_todo = Regex::new(&format!(r"{}( *)TODO ", regex::escape(base))).unwrap();
        let re_done = Regex::new(&format!(r"{}( *)DONE ", regex::escape(base))).unwrap();
        let re_loop = Regex::new(&format!(r"{}( *)LOOP ", regex::escape(base))).unwrap();
        let re_next = Regex::new(&format!(r"{}( *)NEXT ", regex::escape(base))).unwrap();

        if re_todo.is_match(input) {
            return Some(TodoStates::TODO);
        } else if re_done.is_match(input) {
            return Some(TodoStates::DONE);
        } else if re_loop.is_match(input) {
            return Some(TodoStates::LOOP);
        } else if re_next.is_match(input) {
            return Some(TodoStates::NEXT);
        } else {
            None
        }
    }
    fn build(&self) -> String {
        match self {
            TodoStates::DONE => "DONE".to_owned(),
            TodoStates::TODO => "TODO".to_owned(),
            TodoStates::LOOP => "LOOP".to_owned(),
            TodoStates::NEXT => "NEXT".to_owned(),
        }
    }
}

#[derive(Debug)]
pub enum Priority {
    A,
    B,
    C,
}

impl Priority {
    fn value(&self) -> u32 {
        match self {
            Priority::A => 0,
            Priority::B => 1,
            Priority::C => 2,
        }
    }
    fn get(input: &str) -> Option<Priority> {
        let re_a = Regex::new(r"\[#A\]").unwrap();
        let re_b = Regex::new(r"\[#B\]").unwrap();
        let re_c = Regex::new(r"\[#C\]").unwrap();
        if re_a.is_match(&input) {
            return Some(Priority::A);
        } else if re_b.is_match(&input) {
            return Some(Priority::B);
        } else if re_c.is_match(&input) {
            return Some(Priority::C);
        } else {
            return None;
        }
    }
    fn build(&self) -> String {
        match self {
            Priority::A => "[#A]".to_string(),
            Priority::B => "[#B]".to_string(),
            Priority::C => "[#C]".to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Context {
    pub lines: Vec<(ObjectTypes, String)>,
}
impl Context {
    pub fn new() -> Context {
        return Context { lines: Vec::new()};
    }
    pub fn add_context_line(&mut self, line: (ObjectTypes, String)) {
        self.lines.push(line);
    }
}

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
        deadline: Option<ParsedDateTime>,
        scheduled: Option<ParsedDateTime>,
        in_line_scedule: Option<ParsedDateTime>,
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
        context: FileContext,
    }
}

impl ObjectTypes {
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

    fn inline_schedule(input: &str) -> Option<ParsedDateTime> {
        let re_time = Regex::new(r"<[^>]*>").unwrap();
        return match re_time.find(input) {
            Some(mat) => ParsedDateTime::parse(mat.as_str()),
            None => None,
        };
    }

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
    pub fn new_text(input: String) -> ObjectTypes {
        return ObjectTypes::Text { text: input };
    }
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
                    match InfoType::get(mat.as_str()) {
                        Some(value) => info.push(value),
                        None => (),
                    }
                }
            }
            _ => (),
        }
    }
    pub fn new_empty() -> ObjectTypes {
        return ObjectTypes::EmptyLine;
    }
}

#[derive(Debug)]
pub struct File {
    pub context: Context,
    author: Option<String>,
    title: Option<String>,
    pub children: Vec<Object>,
}

impl File {
    pub fn new() -> File {
        return File {
            context: Context { lines: Vec::new() },
            author: None,
            title: None,
            children: Vec::new(),
        };
    }
    pub fn add_author(&mut self, text: &str) {
        let re = regex::Regex::new(r"(?i)#author:\s*(.*)").expect("Could not create Regex");
        if let Some(capture) = re.captures(text) {
            if let Some(author) = capture.get(1) {
                self.author = Some(author.as_str().to_owned());
            }
        }
    }
    pub fn add_title(&mut self, text: &str) {
        let re = regex::Regex::new(r"(?i)#title:\s*(.*)").expect("Could not create Regex");
        if let Some(capture) = re.captures(text) {
            if let Some(title) = capture.get(1) {
                self.title = Some(title.as_str().to_owned());
            }
        }
    }
    pub fn add_children(&mut self, obj: Object) {
        self.children = obj.children;
    }
    pub fn print_children(&self) {
        println!("{:#?}", self.children);
    }
    pub fn update_loop(&mut self) {
        for child in &mut self.children {
            child.update_loop();
        }
    }
}

#[derive(Debug)]
pub struct Object {
    pub object_type: ObjectTypes,
    pub children: Vec<Object>,
}

impl Object {
    pub fn new(obj_type: ObjectTypes) -> Object {
        return {
            Object { object_type: obj_type, children: Vec::new() }
        };
    }
    pub fn empty() -> Object {
        return {
            Object {object_type: ObjectTypes::EmptyLine, children: Vec::new()}
        };
    }
    pub fn add_child(&mut self, obj: Object) {
        self.children.push(obj);
    }
    pub fn build(&self) -> Vec<String> {
        let mut vec = Vec::new();
        vec.push(self.object_type.build());
        vec.extend(self.children.iter().flat_map(|child| child.build()));
        return vec;
    }
    ///First non filtered, Second filtered
    pub fn build_seperate_todo(&self, todostate: &TodoStates, filtered_vec:&mut Vec<String> ) -> Vec<String> {
        let mut std_vec = Vec::new();
        match &self.object_type {
            ObjectTypes::Heading { text, todo, deadline, scheduled, in_line_scedule, value, priority } => match todo {
                Some(todo) if todo == todostate => {
                        filtered_vec.extend(self.build());
                        return Vec::new();
                    }
                _ => (),
            }
            _ => (),
        }
        std_vec.push(self.object_type.build());
        std_vec.extend(self.children.iter().flat_map(|child| child.build_seperate_todo(todostate, filtered_vec)));
        return std_vec;
    }

    pub fn update_loop(&mut self) {
        match &mut self.object_type {
            ObjectTypes::Heading { text, todo, deadline, scheduled, in_line_scedule, value, priority } => {
                if let Some(todo) = todo {
                    if todo == &TodoStates::LOOP {
                        let mut did_update = false;
                        let mut date_sepcified = false;
                        self.update_date(&mut did_update, &mut date_sepcified);
                    }
                }
            }
            _ => (),
        }
        for child in &mut self.children {
            child.update_loop();
        }
    }

    fn update_date(&mut self, did_update: &mut bool, date_specified: &mut bool) {
        for child in self.children.iter_mut() {
            match &mut child.object_type {
                ObjectTypes::INFO { info, text } => {
                    *date_specified = true;
                    for information in info {
                        match information {
                            InfoType::DEADLINE { date } => *did_update = date.update() | *did_update,
                            InfoType::SCHEDULED { date } => *did_update = date.update() | *did_update,
                        }
                    }
                }
                ObjectTypes::ListElement { text, todo, checkbox, value } => {
                    if !*date_specified || *did_update {
                        if let Some(checkbox) = checkbox {
                            *checkbox = false;
                        }
                    }
                }
                _ => (),
            }
            child.update_date(did_update, date_specified);
        }
        // match &mut self.object_type {
        //     ObjectTypes::ListElement { text, todo, checkbox, value } => {
        //         if let Some(checkbox) = checkbox {
        //             *checkbox = *did_update | *checkbox;
        //         }
        //     }
        //     _ => (),
        // }
    }
}
impl Default for Object {
    fn default() -> Self {
        return {
            Object { object_type: ObjectTypes::EmptyLine, children: Vec::new() }
        };
    }
}
