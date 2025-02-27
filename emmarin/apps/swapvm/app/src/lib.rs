use cl::{
    crust::{InputWitness, Nullifier, NullifierSecret, Tx, Unit},
    mantle::ZoneId,
};
use risc0_zkvm::sha::rust_crypto::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const FUNDS_SK: NullifierSecret = NullifierSecret([0; 16]);

// TODO: order pair tokens lexicographically
fn get_pair_share_unit(pair: Pair) -> UnitWitness {
    let mut hasher = Sha256::new();
    hasher.update(b"SWAP_PAIR_SHARE_UNIT");
    hasher.update(&pair.t0);
    hasher.update(&pair.t1);
    UnitWitness {
        spending_covenant: NOP_COVENANT,
        minting_covenant: NOP_COVENANT,
        burning_covenant: NOP_COVENANT,
        arg: hasher.finalize().into(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swap {
    pair: Pair,
    t0_in: u64,
    t1_in: u64,
    t0_out: u64,
    t1_out: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddLiquidity {
    pub pair: Pair,
    pub t0_in: u64,
    pub t1_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveLiquidity {
    _shares: Unit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneData {
    pub nfs: BTreeSet<Nullifier>,
    pub pools: BTreeMap<Pair, Pool>,
    pub zone_id: ZoneId,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    pub t0: Unit,
    pub t1: Unit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub balance_0: u64,
    pub balance_1: u64,
    pub shares_unit: Unit,
    pub total_shares: u64,
}

/// Prove the data was part of the tx output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDataProof;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolsUpdate {
    pub tx: Tx,
    pub notes: Vec<InputWitness>,
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
            * (balance_1_final * 1000 - 3 * swap.t1_in as u128)
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
            ZoneOp::Swap { tx, swap, .. } => self.check_swap(&swap) && self.validate_no_pools(&tx), // TODO: check proof
            ZoneOp::AddLiquidity { tx, .. } => self.validate_no_pools(&tx),
            ZoneOp::RemoveLiquidity { tx, .. } => self.validate_no_pools(&tx), // should we check shares exist?
            ZoneOp::Ledger(tx) => {
                // Just a ledger tx that does not directly interact with the zone,
                // just validate it's not using pool notes
                self.validate_no_pools(tx)
            }
        }
    }

    pub fn pools_update(&mut self, tx: &Tx, notes: &[InputWitness]) {
        // check all previous nullifiers are used
        assert!(self.nfs.iter().all(|nf| tx
            .updates
            .iter()
            .filter(|u| u.zone_id == self.zone_id)
            .flat_map(|u| u.inputs.iter())
            .find(|nf2| *nf2 == nf)
            .is_some()));
        self.nfs.clear();

        // check the exepected pool balances are reflected in the tx outputs
        let outputs = tx
            .updates
            .iter()
            .filter(|u| u.zone_id == self.zone_id)
            .flat_map(|u| u.outputs.iter())
            .collect::<Vec<_>>();

        let expected_pool_balances = self.expected_pool_balances();
        for note in notes {
            assert_eq!(note.nf_sk, FUNDS_SK);
            // TODO: check nonce derivation
            let output = note.to_output();
            let value = expected_pool_balances.get(&output.unit).unwrap();
            assert_eq!(note.value, *value);
            assert!(outputs.contains(&&output.note_commitment()));
            self.nfs.insert(note.nullifier());
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

    pub fn add_liquidity(&mut self, add_liquidity: &AddLiquidity) {
        let pool = self.pools.entry(add_liquidity.pair).or_insert(Pool {
            balance_0: add_liquidity.t0_in,
            balance_1: add_liquidity.t1_in,
            shares_unit: get_pair_share_unit(add_liquidity.pair),
            total_shares: 1,
        });
        let minted_shares = (add_liquidity.t0_in * pool.total_shares / pool.balance_0)
            .min(add_liquidity.t1_in * total_shares / pool.balance_1);
        pool.total_shares += minted_shares; // fix for first deposit
        pool.balance_0 += add_liquidity.t0_in;
        pool.balance_1 += add_liquidity.t1_in;
    }

    pub fn remove_liquidity(&mut self, remove_liquidity: &RemoveLiquidity) {
        let shares = remove_liquidity.shares;
        let pool = self
            .pools
            .iter_mut()
            .find(|(_, pool)| pool.shares_unit == shares)
            .unwrap();
        let t0_out = pool.balance_0 * shares / pool.total_shares;
        let t1_out = pool.balance_1 * shares / pool.total_shares;
        pool.balance_0 -= t0_out;
        pool.balance_1 -= t1_out;
        pool.total_shares -= shares;
    }

    pub fn process_op(&mut self, op: &ZoneOp) {
        match op {
            ZoneOp::Swap { tx, swap, .. } => {
                self.swap(&swap);
                assert!(self.validate_no_pools(&tx);)
                // TODO: check the proof
            }
            ZoneOp::AddLiquidity { tx, add_liquidity } => {
                self.add_liquidity(&add_liquidity);
                assert!(self.validate_no_pools(&tx);)
            }
            ZoneOp::RemoveLiquidity {
                tx,
                remove_liquidity,
            } => {
                self.remove_liquidity(&remove_liquidity);
                assert!(self.validate_no_pools(&tx);)
            }
            ZoneOp::Ledger(tx) => {
                // Just a ledger tx that does not directly interact with the zone,
                // just validate it's not using pool notes
                self.validate_no_pools(tx);
            }
        }
    }

    pub fn update_and_commit(mut self, updates: &PoolsUpdate) -> [u8; 32] {
        self.pools_update(&updates.tx, &updates.notes);
        self.commit()
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
