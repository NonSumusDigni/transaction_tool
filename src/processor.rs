use crate::types::{Client, State, Transaction, TransactionType};

pub fn process_transaction(state: State, transaction: Transaction) -> State {
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
