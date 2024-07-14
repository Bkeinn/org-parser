mod structs;
mod parser;
mod builder;
mod filter;

use clap::{Parser};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::iter::Map;
use std::io::Lines;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(long)]
    file: String,

    #[arg(long)]
    history: String,

    #[arg(long)]
    context: structs::FileContext,
}

fn main() {
    //let args = Args::parse();
    let args = Args{
        file: "todo.org".to_string(),
        history: "history.org".to_string(),
        context: structs::FileContext::Todo,
    };

    let mut lines: Vec<String> = vec![format!("File context: {}", args.context.build())];
    lines.extend(lines_from_file(&args.file));
    let mut org_file = OpenOptions::new().write(true).truncate(true).open(&args.file).expect("No such file found");
    let mut history_file = OpenOptions::new().append(true).open(args.history).expect("No history file found");

    let mut context = structs::Context::new();
    context.parse(lines);
    let mut file = structs::File::new();
    file.add_children(structs::Object::parse(context));
    file.update_loop();
    let mut done_vector = Vec::new();
    let mut cleaned_vector = Vec::new();
    for obj in file.children {
        cleaned_vector.append(&mut obj.build_seperate_todo(&structs::TodoStates::DONE, &mut done_vector));
    }


    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();
    writeln!(history_file, "TIMESTAMP:{}", unix_time).expect("Could not write to history file");
    for line in done_vector {
        writeln!(history_file, "{}", line).expect("Could not write line to history file");
    }
    for line in cleaned_vector {
        writeln!(org_file, "{}", line).expect("Could not write to org file");
    }
}

fn lines_from_file(filename: &str) -> Map<Lines<BufReader<File>>, fn(Result<String, std::io::Error>) -> String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
}
