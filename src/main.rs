use std::{collections::HashMap, env, error::Error, process};

use csv::{ReaderBuilder, Trim};
use types::{Client, State};

mod processor;
mod types;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("Missing filename argument");
        process::exit(1);
    }

    match process_transaction_file(&args[1]) {
        Ok(state) => print_client_state(&state.clients),
        Err(err) => {
            println!("Failed to process '{}': {}", &args[1], err);
            process::exit(1);
        }
    }
}

fn process_transaction_file(path: &String) -> Result<State, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_path(path)?;

    reader.deserialize().try_fold(State::new(), |s, r| {
        Ok(processor::process_transaction(s, r?))
    })
}

fn print_client_state(client_state: &HashMap<u16, Client>) {
    println!("{:?}", client_state)
}
