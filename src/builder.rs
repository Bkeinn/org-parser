use crate::structs::{self, Context, Object, ObjectTypes};

impl structs::File {
    pub fn build_from_context(&mut self) -> Vec<String> {
        self.context
            .lines
            .iter()
            .map(|(obj, _)| obj.build())
            .collect()
    }
}

pub fn build_value(value: u32, repeat: char, end: char) -> String {
    return format!(
        "{}{}",
        (0..value - 1).map(|_| repeat).collect::<String>(),
        end
    );
}

// First String TODO, second string DONE
// pub fn without_done(obj: &Object) -> (Vec<String>, Vec<String>) {
//     if obj.children.is_empty()
// }
