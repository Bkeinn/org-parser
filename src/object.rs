use crate::{object_types,object,  structs::{self, TodoStates}, time_management};

/// This is the main Object information is stored in when parsed into a tree
#[derive(Debug)]
pub struct Object {
    /// Specifies the type of the parent
    object_type: object_types::ObjectTypes,
    /// The childrens underneath it, leaf empty if there is nothing underneath this parent
    children: Vec<Object>,
}

impl Object {
    /// Create an Object with a specific type. If you don't want to specifiy a type, use default() this creates an Object with Emptyline
    pub fn new(obj_type: object_types::ObjectTypes) -> Object {
        return {
            Object { object_type: obj_type, children: Vec::new() }
        };
    }
    /// Children Constructor
    pub fn add_child(&mut self, obj: Object) {
        self.children.push(obj);
    }
    pub fn get_children(self) -> Vec<Object> {
        return self.children;
    }
    /// Creates a Vec<String> representation from Object
    /// I think it's called deserilisation
    pub fn build(&self) -> Vec<String> {
        let mut vec = Vec::new();
        vec.push(self.object_type.build());
        vec.extend(self.children.iter().flat_map(|child| child.build()));
        return vec;
    }
    ///Builds a Vec<String> but filters for Object that are, or are underneath a Heading with specific TodoState
    ///The funktion returns the filtered build of Vec<String> whereas the filtered_vec contains a build representation of the Objects that are or are underneath a filtered Heading
    ///This is mainly used to filter out the Done Todo items
    pub fn build_seperate_todo(&self, todostate: &TodoStates, filtered_vec:&mut Vec<String> ) -> Vec<String> {
        let mut std_vec = Vec::new();
        match &self.object_type {
            object_types::ObjectTypes::Heading { text, todo, deadline, scheduled, in_line_scedule, value, priority } => match todo {
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

    /// Recursivly updtes all items that are or are under a Heading with the TodoState::LOOP
    /// If unsets potential set checkboxes
    /// and if a repeater and a scheduler is given, updates the repeater by how mutch the repeater is set to increase
    pub fn update_loop(&mut self) {
        match &mut self.object_type {
            object_types::ObjectTypes::Heading { text, todo, deadline, scheduled, in_line_scedule, value, priority } => {
                if let Some(todo) = todo {
                    if todo == &structs::TodoStates::LOOP {
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
                object_types::ObjectTypes::INFO { info, text } => {
                    *date_specified = true;
                    for information in info {
                        match information {
                            time_management::InfoType::DEADLINE { date } => *did_update = date.update() | *did_update,
                            time_management::InfoType::SCHEDULED { date } => *did_update = date.update() | *did_update,
                        }
                    }
                }
                object_types::ObjectTypes::ListElement { text, todo, checkbox, value } => {
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
    }
    /// Parses the Context of a file into a tree representation
    /// This converts a Linear representation of the Org mode into a Parent child construct
    pub fn parse(context: structs::Context) -> Object {
        let mut lowest_value = u32::MAX;
        let mut obj_context: structs::Context = structs::Context::new();
        let mut result_obj = Object::default();
        for (number, (obj, string)) in context.lines.into_iter().enumerate() {
            if number == 0 {
                lowest_value = obj.value();
                result_obj = Object::new(obj);
            } else if obj.value() > lowest_value {
                obj_context.add_context_line((obj, string));
            } else {
                if number != 1 {
                    let temp_context = std::mem::take(&mut obj_context);
                result_obj.add_child(object::Object::parse(temp_context));
                }
                lowest_value = obj.value();
                obj_context.add_context_line((obj, string));
            }
        }
        if !obj_context.lines.is_empty() {
            result_obj.add_child(object::Object::parse(obj_context));
        }
        return result_obj;
    }
}
impl Default for Object {
    fn default() -> Self {
        return {
            Object { object_type: object_types::ObjectTypes::EmptyLine, children: Vec::new() }
        };
    }
}
