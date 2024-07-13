mod builder;
mod parser;
mod structs;

//  let lines: Vec<String> = input.lines().map(|line| line.to_string()).collect();

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        str::FromStr,
    };

    use chrono::NaiveDate;
    use structs::Object;

    use super::*;

    #[test]
    fn raw_parse_into_tree() {
        let file = File::open("todo.org").expect("No such file found");
        let buf = BufReader::new(file);
        let lines: Vec<String> = buf.lines().map(|l| l.expect("No Line")).collect();

        let mut file = structs::File::new();
        file.parse(lines);
        println!("{:#?}", file.context);

        for i in file.build_from_context() {
            println!("{i}");
        }

        file.add_children(Object::parse(file.context));
        file.print_children();
    }

    #[test]
    fn time_parse() {
        let time = structs::ParsedDateTime::parse("<2024-07-12 Fri>").unwrap();
        println!("{:#?}", time);
        let time = structs::ParsedDateTime::parse("<2024-07-12 Fri .+ld>").unwrap();
        println!("{:#?}", time);
        panic!("Stop");
    }
    #[test]
    fn info_parse() {
        let info = structs::ObjectTypes::new_info(
            "DEADLINE: <2024-07-13 Sat> SCHEDULED: <2024-07-12 Fri>".to_string(),
        );
        panic!("Stop");
    }
}
