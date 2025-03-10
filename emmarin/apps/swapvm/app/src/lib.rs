use cl::{
    crust::{
        balance::{UnitWitness, NOP_COVENANT},
        InputWitness, Nonce, Nullifier, NullifierCommitment, NullifierSecret, OutputWitness, Tx,
        Unit,
    },
    mantle::ZoneId,
};
use rand::RngCore;
use risc0_zkvm::sha::rust_crypto::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const FUNDS_SK: NullifierSecret = NullifierSecret([0; 16]);
pub const ZONE_ID: [u8; 32] = [128; 32];

pub fn swap_goal_unit() -> UnitWitness {
    UnitWitness::nop(b"SWAP")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwapOutput {
    // value will be set at the market price
    pub state: [u8; 32],
    pub unit: Unit,
    pub nonce: Nonce,
    pub zone_id: ZoneId,
    pub nf_pk: NullifierCommitment,
}
impl SwapOutput {
    pub fn basic(
        unit: Unit,
        zone_id: ZoneId,
        nf_pk: NullifierCommitment,
        rng: impl RngCore,
    ) -> Self {
        Self {
            state: [0; 32],
            unit,
            nonce: Nonce::random(rng),
            zone_id,
            nf_pk,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapArgs {
    // the user specifies the template forthe output note
    pub output: SwapOutput,
    // minimum value of the output note
    pub limit: u64,
}

impl SwapArgs {
    pub fn to_output(self, value: u64) -> OutputWitness {
        assert!(value >= self.limit);
        OutputWitness {
            state: self.output.state,
            value,
            unit: self.output.unit,
            nonce: self.output.nonce,
            zone_id: self.output.zone_id,
            nf_pk: self.output.nf_pk,
        }
    }
}

pub fn swap_goal_note(rng: impl RngCore) -> OutputWitness {
    OutputWitness {
        state: [0u8; 32],
        value: 1,
        unit: swap_goal_unit().unit(),
        nonce: Nonce::random(rng),
        zone_id: ZONE_ID,
        nf_pk: NullifierSecret::zero().commit(),
    }
}

// TODO: order pair tokens lexicographically
fn get_pair_share_unit(pair: Pair) -> UnitWitness {
    let mut hasher = Sha256::new();
    hasher.update(b"SWAP_PAIR_SHARE_UNIT");
    hasher.update(pair.t0);
    hasher.update(pair.t1);
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
    pub pk_out: NullifierCommitment,
    pub nonce: Nonce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharesToMint {
    pub amount: u64,
    pub unit: Unit,
    pub pk_out: NullifierCommitment,
    pub nonce: Nonce,
}

impl SharesToMint {
    pub fn to_output(&self, zone_id: ZoneId) -> OutputWitness {
        OutputWitness {
            state: [0; 32],
            value: self.amount,
            unit: self.unit,
            nonce: self.nonce,
            zone_id,
            nf_pk: self.pk_out,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveLiquidity {
    shares: InputWitness,
    nf_pk: NullifierCommitment,
    nonce: Nonce,
}

impl RemoveLiquidity {
    pub fn to_output(&self, value: u64, unit: Unit, zone_id: ZoneId) -> OutputWitness {
        OutputWitness {
            state: [0; 32],
            value,
            unit,
            nonce: self.nonce,
            zone_id,
            nf_pk: self.nf_pk,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneData {
    pub nfs: BTreeSet<Nullifier>,
    pub pools: BTreeMap<Pair, Pool>,
    pub zone_id: ZoneId,
    pub shares_to_mint: Vec<SharesToMint>,
    pub shares_to_redeem: Vec<OutputWitness>,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
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
    Swap(Swap),
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

/// This contains the changes at the ledger level that can only be done by the executor
/// because they are dependant on the order of transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    pub tx: Tx,
    /// Update the balance of the pool
    pub pool_notes: Vec<InputWitness>,
}

// Txs are of the following form:
impl ZoneData {
    /// A swap does not need to directly modify the pool balances, but the executor
    /// should make sure that required funds are provided.
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
        let Some(zone_update) = tx.updates.get(&self.zone_id) else {
            // this tx is not involving this zone, therefore it is
            // guaranteed to not consume pool notes
            return true;
        };

        self.nfs.iter().all(|nf| !zone_update.has_input(nf))
    }

    pub fn validate_op(&self, op: &ZoneOp) -> bool {
        match op {
            ZoneOp::Swap(swap) => self.check_swap(swap),
            ZoneOp::AddLiquidity { tx, .. } => self.validate_no_pools(tx),
            ZoneOp::RemoveLiquidity { tx, .. } => self.validate_no_pools(tx), // should we check shares exist?
            ZoneOp::Ledger(tx) => {
                // Just a ledger tx that does not directly interact with the zone,
                // just validate it's not using pool notes
                self.validate_no_pools(tx)
            }
        }
    }

    pub fn pools_update(&mut self, tx: &Tx, pool_notes: &[InputWitness]) {
        let Some(zone_update) = tx.updates.get(&self.zone_id) else {
            // The tx is not involving this zone, nothing to do.
            return;
        };
        // check all previous nullifiers are used
        assert!(self.nfs.iter().all(|nf| zone_update.has_input(nf)));
        self.nfs.clear();

        // check the exepected pool balances are reflected in the tx outputs
        let expected_pool_balances = self.expected_pool_balances();
        for note in pool_notes {
            assert_eq!(note.nf_sk, FUNDS_SK);
            // TODO: check nonce derivation
            let output = note.to_output();
            let value = expected_pool_balances.get(&output.unit).unwrap();
            assert_eq!(note.value, *value);

            assert!(zone_update.has_output(&output.note_commitment()));
            self.nfs.insert(note.nullifier());
        }
    }

    pub fn check_minted_shares(&self, tx: &Tx) {
        let Some(zone_update) = tx.updates.get(&self.zone_id) else {
            return;
        };

        for shares in &self.shares_to_mint {
            let output = shares.to_output(self.zone_id);
            assert!(zone_update.has_output(&output.note_commitment()));
        }
    }

    pub fn check_redeemed_shares(&self, tx: &Tx) {
        let Some(zone_update) = tx.updates.get(&self.zone_id) else {
            return;
        };

        for shares in &self.shares_to_redeem {
            assert!(zone_update.has_output(&shares.note_commitment()));
        }

        // TODO: check shares have been burned
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
            shares_unit: get_pair_share_unit(add_liquidity.pair).unit(),
            total_shares: 1,
        });
        let minted_shares = (add_liquidity.t0_in * pool.total_shares / pool.balance_0)
            .min(add_liquidity.t1_in * pool.total_shares / pool.balance_1);
        pool.total_shares += minted_shares; // fix for first deposit
        pool.balance_0 += add_liquidity.t0_in;
        pool.balance_1 += add_liquidity.t1_in;

        self.shares_to_mint.push(SharesToMint {
            amount: minted_shares,
            unit: pool.shares_unit,
            pk_out: add_liquidity.pk_out,
            nonce: add_liquidity.nonce,
        });
    }

    pub fn remove_liquidity(&mut self, remove_liquidity: &RemoveLiquidity) {
        let shares = remove_liquidity.shares;
        let (pair, pool) = self
            .pools
            .iter_mut()
            .find(|(_, pool)| pool.shares_unit == shares.unit_witness.unit())
            .unwrap();
        let t0_out = pool.balance_0 * shares.value / pool.total_shares;
        let t1_out = pool.balance_1 * shares.value / pool.total_shares;
        pool.balance_0 -= t0_out;
        pool.balance_1 -= t1_out;
        pool.total_shares -= shares.value;

        self.shares_to_redeem
            .push(remove_liquidity.to_output(t0_out, pair.t0, self.zone_id));

        self.shares_to_redeem
            .push(remove_liquidity.to_output(t1_out, pair.t1, self.zone_id));
    }

    pub fn process_op(&mut self, op: &ZoneOp) {
        match op {
            ZoneOp::Swap(swap) => self.swap(swap),
            ZoneOp::AddLiquidity {
                tx, add_liquidity, ..
            } => {
                self.add_liquidity(add_liquidity);
                assert!(self.validate_no_pools(tx));
                // TODo: check proof
            }
            ZoneOp::RemoveLiquidity {
                tx,
                remove_liquidity,
                ..
            } => {
                self.remove_liquidity(remove_liquidity);
                assert!(self.validate_no_pools(tx));
                // TODO: check proof
            }
            ZoneOp::Ledger(tx) => {
                // Just a ledger tx that does not directly interact with the zone,
                // just validate it's not using pool notes
                self.validate_no_pools(tx);
            }
        }
    }

    pub fn update_and_commit(mut self, updates: &StateUpdate) -> [u8; 32] {
        self.pools_update(&updates.tx, &updates.pool_notes);
        self.check_minted_shares(&updates.tx);
        self.check_redeemed_shares(&updates.tx);
        self.commit()
    }

    pub fn commit(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for nf in &self.nfs {
            hasher.update(nf);
        }
        for (pair, pool) in self.pools.iter() {
            hasher.update(pair.t0);
            hasher.update(pair.t1);
            hasher.update(pool.balance_0.to_le_bytes());
            hasher.update(pool.balance_1.to_le_bytes());
        }
        hasher.update(self.zone_id);
        hasher.finalize().into()
    }
}
