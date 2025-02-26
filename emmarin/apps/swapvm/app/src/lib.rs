use cl::{
    crust::{
        balance::{UnitWitness, NOP_COVENANT},
        Nullifier, Tx, Unit,
    },
    mantle::{ledger::Ledger, ZoneId, ZoneState},
};
use risc0_zkvm::sha::rust_crypto::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const SWAP_GOAL_UNIT: UnitWitness = UnitWitness {
    spending_covenant: NOP_COVENANT,
    minting_covenant: NOP_COVENANT,
    burning_covenant: NOP_COVENANT,
};

pub struct SwapVmPrivate {
    pub old: ZoneState,
    pub new_ledger: Ledger,
    pub data: ZoneData,
}

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
        proof: OutputDataProof,
    },
    AddLiquidity {
        tx: Tx,
        add_liquidity: AddLiquidity,
        proof: OutputDataProof,
    },
    RemoveLiquidity {
        tx: Tx,
        remove_liquidity: RemoveLiquidity,
        proof: OutputDataProof,
    },
    Ledger(Tx),
}

// Txs are of the following form:
impl ZoneData {
    pub fn swap(&mut self, swap: &Swap) {
        assert!(self.check_swap(swap));
        let pool = self.pools.get_mut(&swap.pair).unwrap();
        pool.balance_0 += swap.t0_in - swap.t0_out;
        pool.balance_1 += swap.t1_in - swap.t1_out;
    }

    pub fn check_swap(&self, swap: &Swap) -> bool {
        let Some(pool) = self.pools.get(&swap.pair) else {
            return false;
        };

        let balance_0_start = pool.balance_0 as u128;
        let balance_1_start = pool.balance_1 as u128;
        let balance_0_final = balance_0_start + swap.t0_in as u128 - swap.t0_out as u128;
        let balance_1_final = balance_1_start + swap.t1_in as u128 - swap.t1_out as u128;

        (balance_0_final * 1000 - 3 * swap.t0_in as u128)
            * (balance_0_final * 1000 - 3 * swap.t1_in as u128)
            == balance_0_start * balance_1_start
    }

    /// Check no pool notes are used in this tx
    pub fn validate_no_pools(&self, tx: &Tx) -> bool {
        tx.updates
            .iter()
            .filter(|u| u.zone_id == self.zone_id)
            .flat_map(|u| u.inputs.iter())
            .all(|nf| !self.nfs.contains(nf))
    }

    pub fn validate_op(&self, op: &ZoneOp) -> bool {
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

    pub fn expected_pool_balances(&self) -> BTreeMap<Unit, u64> {
        let mut expected_pool_balances = BTreeMap::new();
        for (Pair { t0, t1 }, pool) in self.pools.iter() {
            *expected_pool_balances.entry(*t0).or_insert(0) += pool.balance_0;
            *expected_pool_balances.entry(*t1).or_insert(0) += pool.balance_1;
        }

        expected_pool_balances
    }

    pub fn process_op(&mut self, op: &ZoneOp) {
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
                todo!()
            }
            ZoneOp::RemoveLiquidity {
                tx,
                remove_liquidity,
                proof,
            } => {
                todo!()
            }
            ZoneOp::Ledger(tx) => {
                // Just a ledger tx that does not directly interact with the zone,
                // just validate it's not using pool notes
                self.validate_no_pools(tx);
            }
        }
    }

    pub fn commit(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for nf in &self.nfs {
            hasher.update(nf);
        }
        for (pair, pool) in self.pools.iter() {
            hasher.update(&pair.t0);
            hasher.update(&pair.t1);
            hasher.update(&pool.balance_0.to_le_bytes());
            hasher.update(&pool.balance_1.to_le_bytes());
        }
        hasher.update(&self.zone_id);
        hasher.finalize().into()
    }
}
