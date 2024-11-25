use cl::cl::{Balance, BalanceWitness};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BalancePublic {
    pub balances: Vec<Balance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BalancePrivate {
    pub balances: Vec<BalanceWitness>,
}
