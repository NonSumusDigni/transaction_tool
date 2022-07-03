use std::{collections::HashMap, env, error::Error, process};

use csv::{ReaderBuilder, Trim};
use serde::{de, Deserialize, Serialize};

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Deserialize, Debug)]
struct Transaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,

    #[serde(rename = "client")]
    client_id: u16,

    #[serde(rename = "tx")]
    id: u32,

    #[serde(deserialize_with = "optional_amount_value")]
    amount: f64,

    #[serde(default)]
    disputed: bool,
}

#[derive(Serialize, Debug)]
struct Client {
    id: u16,
    locked: bool,
    available: f64,
    held: f64,
    total: f64,
}

impl Client {
    fn new(id: u16) -> Self {
        Client {
            id,
            locked: false,
            available: 0.0,
            held: 0.0,
            total: 0.0,
        }
    }
}

#[derive(Debug)]
struct State {
    transfers: HashMap<u32, Transaction>,
    clients: HashMap<u16, Client>,
}

impl State {
    fn new() -> Self {
        Self {
            transfers: HashMap::new(),
            clients: HashMap::new(),
        }
    }
}

// since amount can be blank for some transaction types,
// this is a custom deserializer fn to handle the empty string case
fn optional_amount_value<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    match s.parse::<f64>() {
        Ok(f) => Ok(f),
        Err(_) => Ok(0.0),
    }
}

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

    reader
        .deserialize()
        .try_fold(State::new(), |s, r| Ok(process_transaction(s, r?)))
}

fn print_client_state(client_state: &HashMap<u16, Client>) {
    println!("{:?}", client_state)
}

fn process_transaction(state: State, transaction: Transaction) -> State {
    match transaction.transaction_type {
        TransactionType::Deposit => process_deposit(state, transaction),
        TransactionType::Withdrawal => process_withdrawal(state, transaction),
        TransactionType::Dispute => process_dispute(state, transaction),
        TransactionType::Resolve => process_resolve(state, transaction),
        TransactionType::Chargeback => process_chargeback(state, transaction),
    }
}

fn process_deposit(mut state: State, transaction: Transaction) -> State {
    // if this deposit references an already existing transaction id, it is invalid and should be skipped
    if state.transfers.contains_key(&transaction.id) {
        return state;
    }

    let client = match state.clients.get_mut(&transaction.client_id) {
        Some(client) => client,
        None => {
            state
                .clients
                .insert(transaction.client_id, Client::new(transaction.client_id));
            state.clients.get_mut(&transaction.client_id).unwrap()
        }
    };

    if client.locked {
        return state;
    }

    client.available += transaction.amount;
    client.total += transaction.amount;

    state.transfers.insert(transaction.id, transaction);

    state
}

fn process_withdrawal(mut state: State, transaction: Transaction) -> State {
    // if this withdrawal references an already existing transaction id, it is invalid and should be skipped
    if state.transfers.contains_key(&transaction.id) {
        return state;
    }

    let client = match state.clients.get_mut(&transaction.client_id) {
        Some(client) => client,
        None => return state, // client doesn't exist, withdrawal is invalid
    };

    if client.locked || client.available < transaction.amount {
        return state;
    }

    client.available -= transaction.amount;
    client.total -= transaction.amount;

    state.transfers.insert(transaction.id, transaction);

    state
}

fn process_dispute(mut state: State, transaction: Transaction) -> State {
    let mut target_transaction = match state.transfers.get_mut(&transaction.id) {
        Some(tx) => tx,
        None => return state,
    };

    if target_transaction.disputed
        || target_transaction.transaction_type != TransactionType::Deposit
        || target_transaction.client_id != transaction.client_id
    {
        return state;
    }

    let mut client = state
        .clients
        .get_mut(&target_transaction.client_id)
        .unwrap();

    if client.locked {
        return state;
    }

    target_transaction.disputed = true;
    client.held += target_transaction.amount;
    client.available -= target_transaction.amount;

    state
}

fn process_resolve(mut state: State, transaction: Transaction) -> State {
    let mut target_transaction = match state.transfers.get_mut(&transaction.id) {
        Some(tx) => tx,
        None => return state,
    };

    if !target_transaction.disputed || target_transaction.client_id != transaction.client_id {
        return state;
    }

    let mut client = state
        .clients
        .get_mut(&target_transaction.client_id)
        .unwrap();

    if client.locked {
        return state;
    }

    target_transaction.disputed = false;
    client.held -= target_transaction.amount;
    client.available += target_transaction.amount;

    state
}

fn process_chargeback(mut state: State, transaction: Transaction) -> State {
    let target_transaction = match state.transfers.get(&transaction.id) {
        Some(tx) => tx,
        None => return state,
    };

    if !target_transaction.disputed || target_transaction.client_id != transaction.client_id {
        return state;
    }

    let mut client = state
        .clients
        .get_mut(&target_transaction.client_id)
        .unwrap();

    if client.locked {
        return state;
    }

    client.locked = true;
    client.held -= target_transaction.amount;
    client.total -= target_transaction.amount;

    state
}
