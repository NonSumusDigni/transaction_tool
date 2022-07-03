use std::{collections::HashMap, env, error::Error, io, process};

use csv::{ReaderBuilder, Trim, Writer};
use types::{Client, State};

mod processor;
mod types;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("Missing filename argument");
        process::exit(1);
    }

    if let Err(err) = try_main(&args[1]) {
        eprintln!("Failed to process '{}': {}", &args[1], err);
        process::exit(1);
    }
}

fn try_main(path: &String) -> Result<(), Box<dyn Error>> {
    let state = process_transaction_file(path)?;
    print_client_state(&state.clients)?;

    Ok(())
}

fn process_transaction_file(path: &String) -> Result<State, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_path(path)?;

    reader.deserialize().try_fold(State::new(), |s, r| {
        Ok(processor::process_transaction(s, r?))
    })
}

fn print_client_state(client_state: &HashMap<u16, Client>) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_writer(io::stdout());

    for client in client_state.values() {
        writer.serialize(client)?;
    }

    writer.flush()?;

    Ok(())
}
