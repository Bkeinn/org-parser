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

mod object;
mod object_types;
mod time_management;

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
    /// Scripting for org files
    /// Short explanation what is happening
    let args = Args::parse(); // Read in the Arguments

    let mut lines: Vec<String> = vec![format!("File context: {}", args.context.build())]; // Creates the Vec<String> with the specific context
    lines.extend(lines_from_file(&args.file)); // Reads in the org file
    let mut org_file = OpenOptions::new().write(true).truncate(true).open(&args.file).expect("No such file found"); //Open the org file, so that it can be rewritten
    let mut history_file = OpenOptions::new().append(true).open(args.history).expect("No history file found");// Opens the history file

    let mut context = structs::Context::new(); // Creates new context Object
    context.parse(lines);// Parses the raw lines into ObjectTypes -> The program now knows what they are
    let mut file = structs::File::new(); // Creates a virtual org file
    file.add_children(object::Object::parse(context)); // Parses the context into the org file, so that a tree structure gets created
    file.update_loop(); // Updtes all the Headers with the LOOP state
    let mut done_vector = Vec::new();
    let mut cleaned_vector = Vec::new();
    for obj in file.children { // goes through the virtual org file and seperates it into two piles, the ones under a DONE Header and the rest
        cleaned_vector.append(&mut obj.build_seperate_todo(&structs::TodoStates::DONE, &mut done_vector));
    }


    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();
    writeln!(history_file, "TIMESTAMP:{}", unix_time).expect("Could not write to history file"); // Appends the unix time stamp to the History file
    for line in done_vector { // Writes done to history file
        writeln!(history_file, "{}", line).expect("Could not write line to history file");
    }
    for line in cleaned_vector { // Writes rest to org file
        writeln!(org_file, "{}", line).expect("Could not write to org file");
    }
}

fn lines_from_file(filename: &str) -> Map<Lines<BufReader<File>>, fn(Result<String, std::io::Error>) -> String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
}
