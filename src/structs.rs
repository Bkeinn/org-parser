use crate::{object, object_types};
use std::{default, u32::MAX};
use clap::ValueEnum;


/// Constructor funktions will allways require the whole Line, not just snipets
use regex::{self, Regex, Replacer};

#[derive(Debug, Clone, Copy, ValueEnum)]
/// Org is a Context dependant language, so this is a place to add potetial contexts.
pub enum FileContext {
    Todo,
}

#[derive(Debug, PartialEq)]
pub enum TodoStates {
    TODO,
    DONE,
    LOOP,
    NEXT,
}

#[derive(Debug)]
pub enum Priority {
    A,
    B,
    C,
}


#[derive(Debug, Default)]
pub struct Context {
    pub lines: Vec<(object_types::ObjectTypes, String)>,
}

#[derive(Debug)]
pub struct File {
    pub context: Context,
    author: Option<String>,
    title: Option<String>,
    pub children: Vec<object::Object>,
}

impl FileContext {
    pub fn build(&self) -> &str {
        match self {
            FileContext::Todo => "Todo file",
        }
    }
}

impl TodoStates {
    pub fn get(base: &str, input: &str) -> Option<TodoStates> {
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

    /// Creates a String representation of Todo
    pub fn build(&self) -> String {
        match self {
            TodoStates::DONE => "DONE".to_owned(),
            TodoStates::TODO => "TODO".to_owned(),
            TodoStates::LOOP => "LOOP".to_owned(),
            TodoStates::NEXT => "NEXT".to_owned(),
        }
    }
}


impl Priority {
    /// Gives values to the priorities if you would have to sort by priority
    fn value(&self) -> u32 {
        match self {
            Priority::A => 0,
            Priority::B => 1,
            Priority::C => 2,
        }
    }
    pub fn get(input: &str) -> Option<Priority> {
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
    pub fn build(&self) -> String {
        match self {
            Priority::A => "[#A]".to_string(),
            Priority::B => "[#B]".to_string(),
            Priority::C => "[#C]".to_string(),
        }
    }
}

impl Context {
    pub fn new() -> Context {
        return Context { lines: Vec::new()};
    }
    pub fn add_context_line(&mut self, line: (object_types::ObjectTypes, String)) {
        self.lines.push(line);
    }
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
    pub fn add_children(&mut self, obj: object::Object) {
        self.children = obj.get_children();
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
