use cl::crust::{balance::UnitWitness, Nullifier, Tx, Unit};
use std::collections::{BTreeMap, BTreeSet};

const SWAP_GOAL_UNIT: UnitWitness = UnitWitness {
    spending_covenant: NOP_COVENANT,
    minting_covenant: NOP_COVENANT,
    burning_covenant: NOP_COVENANT,
};

pub struct Swap {
    pair: Pair,
    t0_in: u64,
    t1_in: u64,
    t0_out: u64,
    t1_out: u64,
}

pub struct AddLiquidity {
    pair: Pair,
    t0_in: u64,
    t1_in: u64,
}

pub struct RemoveLiquidity {
    shares: Unit,
}

pub struct ZoneData {
    nfs: BTreeSet<Nullifier>,
    pools: BTreeMap<Pair, Pool>,
    zone_id: ZoneId,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Pair {
    pub t0: Unit,
    pub t1: Unit,
}

pub struct Pool {
    pub balance_0: u64,
    pub balance_1: u64,
}

/// Prove the data was part of the tx output
pub struct OutputDataProof;

pub enum ZoneOp {
    Swap {
        tx: Tx,
        swap: Swap,
        proof: DataProof,
    },
    AddLiquidity {
        tx: Tx,
        add_liquidity: AddLiquidity,
        proof: DataProof,
    },
    RemoveLiquidity {
        tx: Tx,
        remove_liquidity: RemoveLiquidity,
        proof: DataProof,
    },
    UpdatePool {
        tx: Tx,
        pool: Pool,
        proof: DataProof,
    },
    Ledger(Tx),
}

// Txs are of the following form:
impl ZoneData {
    pub fn add_liquidity(&mut self) {}

    pub fn remove_liquidity(&mut self) {}

    pub fn swap(&mut self, swap: &Swap) {
        assert!(self.check_swap(swap));
        let pool = self.pools.get_mut(&swap.pair).unwrap();
        pool.balance_0 += swap.t0_in - swap.t0_out;
        pool.balance_1 += swap.t1_in - swap.t1_out;
    }

    pub fn check_swap(&self, swap: &Swap) -> bool {
        let pool = self.pools.get(&swap.pair) else {
            return false;
        };

        let balance_0_start = pool.balance_0 as u128;
        let balance_1_start = pool.balance_1 as u128;
        let balance_0_final = balance_0_start + swap.t0_in as u128 - swap.t0_out as u128;
        let balance_1_final = balance_1_start + swap.t1_in as u128 - swap.t1_out as u128;

        (balance_0_final * 1000 - 3 * swap.t0_in as u128)
            * (balance_0_final * 1000 - 3 * swap.t1_in as u128)
            == balance_0_start * balance_1_start;
    }

    /// Check no pool notes are used in this tx
    pub fn validate_no_pools(&self, tx: &Tx) -> bool {
        tx.inputs.iter().all(|input| !self.nfs.contains(&input.nf))
    }

    pub fn validate_op(&self, op: ZoneOp) -> bool {
        match op {
            ZoneOp::Swap { tx, swap, proof } => {
                self.check_swap(&swap) && self.validate_no_pools(&tx)
            }
            ZoneOp::AddLiquidity { tx, .. } => self.validate_no_pools(&tx),
            ZoneOp::RemoveLiquidity { tx, .. } => self.validate_no_pools(&tx), // should we check shares exist?
            ZoneOp::Ledger(tx) => {
                // Just a ledger tx that does not directly interact with the zone,
                // just validate it's not using pool notes
                self.validate_no_pools(tx)
            }
        }
    }

    pub fn process_op(&mut self, op: ZoneOp) {
        match op {
            ZoneOp::Swap { tx, swap, proof } => {
                self.swap(&swap);
                self.validate_no_pools(&tx);
            }
            ZoneOp::AddLiquidity {
                tx,
                add_liquidity,
                proof,
            } => {
                self.add_liquidity(&add_liquidity);
            }
            ZoneOp::RemoveLiquidity {
                tx,
                remove_liquidity,
                proof,
            } => {
                self.remove_liquidity(&remove_liquidity);
            }
            ZoneOp::Ledger(tx) => {
                // Just a ledger tx that does not directly interact with the zone,
                // just validate it's not using pool notes
                self.validate_no_pools(tx);
            }
            ZoneOp::UpdatePool { tx, pool, proof } => {
                todo!()
            }
        }
    }
}
