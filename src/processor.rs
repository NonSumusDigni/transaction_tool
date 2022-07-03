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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_deposit() {
        let start_state = State::new();
        let tx = Transaction {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            id: 1,
            amount: 1.0,
            disputed: false,
        };

        let result_state = process_transaction(start_state, tx);

        assert_eq!(result_state.clients.len(), 1);
        assert!(result_state.clients.contains_key(&1));

        let result_client = result_state.clients.get(&1).unwrap();

        assert_eq!(result_client.available, 1.0);
        assert_eq!(result_client.total, 1.0);
        assert_eq!(result_client.held, 0.0);
    }

    #[test]
    fn valid_withdrawal() {
        let mut state = State::new();
        let txs = vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client_id: 1,
                id: 1,
                amount: 1.0,
                disputed: false,
            },
            Transaction {
                transaction_type: TransactionType::Withdrawal,
                client_id: 1,
                id: 2,
                amount: 0.35,
                disputed: false,
            },
        ];

        for tx in txs {
            state = process_transaction(state, tx);
        }

        assert_eq!(state.clients.len(), 1);

        let result_client = state.clients.get(&1).unwrap();

        assert_eq!(result_client.available, 0.65);
        assert_eq!(result_client.total, 0.65);
    }

    #[test]
    fn invalid_withdrawal_insufficient_funds() {
        let mut state = State::new();
        let txs = vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client_id: 1,
                id: 1,
                amount: 1.0,
                disputed: false,
            },
            Transaction {
                transaction_type: TransactionType::Withdrawal,
                client_id: 1,
                id: 2,
                amount: 10.0,
                disputed: false,
            },
        ];

        for tx in txs {
            state = process_transaction(state, tx);
        }

        assert_eq!(state.clients.len(), 1);

        let result_client = state.clients.get(&1).unwrap();

        assert_eq!(result_client.available, 1.0);
        assert_eq!(result_client.total, 1.0);
    }

    #[test]
    fn dispute_and_resolve() {
        let mut state = State::new();
        let txs_1 = vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client_id: 1,
                id: 1,
                amount: 1.0,
                disputed: false,
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client_id: 1,
                id: 1,
                amount: 0.0,
                disputed: false,
            },
        ];

        for tx in txs_1 {
            state = process_transaction(state, tx);
        }

        assert_eq!(state.clients.len(), 1);

        let mut result_client = state.clients.get(&1).unwrap();

        assert_eq!(result_client.available, 0.0);
        assert_eq!(result_client.total, 1.0);
        assert_eq!(result_client.held, 1.0);

        let resolve_tx = Transaction {
            transaction_type: TransactionType::Resolve,
            client_id: 1,
            id: 1,
            amount: 0.0,
            disputed: false,
        };

        state = process_transaction(state, resolve_tx);

        result_client = state.clients.get(&1).unwrap();

        assert_eq!(result_client.available, 1.0);
        assert_eq!(result_client.total, 1.0);
        assert_eq!(result_client.held, 0.0);
    }

    #[test]
    fn chargeback() {
        let mut state = State::new();
        let txs = vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client_id: 1,
                id: 1,
                amount: 1.0,
                disputed: false,
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client_id: 1,
                id: 1,
                amount: 0.0,
                disputed: false,
            },
            Transaction {
                transaction_type: TransactionType::Chargeback,
                client_id: 1,
                id: 1,
                amount: 0.0,
                disputed: false,
            },
        ];

        for tx in txs {
            state = process_transaction(state, tx);
        }

        assert_eq!(state.clients.len(), 1);

        let result_client = state.clients.get(&1).unwrap();

        assert_eq!(result_client.available, 0.0);
        assert_eq!(result_client.total, 0.0);
        assert_eq!(result_client.held, 0.0);
        assert!(result_client.locked);
    }

    #[test]
    fn invalid_withdrawal_no_client() {
        let start_state = State::new();
        let tx = Transaction {
            transaction_type: TransactionType::Withdrawal,
            client_id: 1,
            id: 1,
            amount: 1.0,
            disputed: false,
        };

        let result_state = process_transaction(start_state, tx);

        assert!(result_state.clients.is_empty());
    }
}
