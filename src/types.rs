use std::collections::HashMap;

use serde::{de, Deserialize, Serialize};

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,

    #[serde(rename = "client")]
    pub client_id: u16,

    #[serde(rename = "tx")]
    pub id: u32,

    #[serde(deserialize_with = "optional_amount_value")]
    pub amount: f64,

    #[serde(default)]
    pub disputed: bool,
}

#[derive(Serialize, Debug)]
pub struct Client {
    pub id: u16,
    pub locked: bool,
    pub available: f64,
    pub held: f64,
    pub total: f64,
}

impl Client {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            locked: false,
            available: 0.0,
            held: 0.0,
            total: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct State {
    pub transfers: HashMap<u32, Transaction>,
    pub clients: HashMap<u16, Client>,
}

impl State {
    pub fn new() -> Self {
        Self {
            transfers: HashMap::new(),
            clients: HashMap::new(),
        }
    }
}

// since amount can be blank for some transaction types,
// this is a custom deserializer fn to handle the empty string case
pub fn optional_amount_value<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    match s.parse::<f64>() {
        Ok(f) => Ok(f),
        Err(_) => Ok(0.0),
    }
}
