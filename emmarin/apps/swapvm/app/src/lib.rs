use cl::{
    crust::{
        balance::{UnitWitness, NOP_COVENANT},
        tx::LedgerUpdate,
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
    // the nonce used in the swap goal note
    pub nonce: Nonce,
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

pub fn swap_goal_note(nonce: Nonce) -> OutputWitness {
    OutputWitness {
        state: [0u8; 32],
        value: 1,
        unit: swap_goal_unit().unit(),
        nonce,
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
pub struct ZoneData {
    pub nfs: BTreeSet<Nullifier>,
    pub pools: BTreeMap<Pair, Pool>,
    pub zone_id: ZoneId,
    pub swaps_output: Vec<OutputWitness>,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    pub t0: Unit,
    pub t1: Unit,
}

impl Pair {
    pub fn new(t_a: Unit, t_b: Unit) -> Self {
        Self {
            t0: std::cmp::min(t_a, t_b),
            t1: std::cmp::max(t_a, t_b),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub balance_0: u64,
    pub balance_1: u64,
    pub shares_unit: Unit,
    pub total_shares: u64,
}

impl Pool {
    pub fn price(&self) -> f64 {
        self.balance_1 as f64 / self.balance_0 as f64
    }
}

// Txs are of the following form:
impl ZoneData {
    pub fn new() -> Self {
        Self {
            nfs: Default::default(),
            pools: Default::default(),
            zone_id: ZONE_ID,
            swaps_output: Default::default(),
        }
    }

    pub fn pair_price(&self, t_in: Unit, t_out: Unit) -> Option<f64> {
        let pair = Pair::new(t_in, t_out);
        self.pools.get(&pair).map(|pool| pool.price()).map(|price| {
            if t_in == pair.t0 {
                price
            } else {
                1.0 / price
            }
        })
    }

    pub fn amount_out(&self, t_in: Unit, t_out: Unit, amount_in: u64) -> Option<u64> {
        let pair = Pair::new(t_in, t_out);
        let pool = self.pools.get(&pair)?;

        let (balance_in, balance_out) = if pair.t0 == t_in {
            (pool.balance_0, pool.balance_1)
        } else {
            (pool.balance_1, pool.balance_0)
        };

        let amount_in_after_fee = amount_in * 1000 - amount_in * 3;

        let amount_out =
            (balance_out * 1000 * amount_in_after_fee) / (balance_in * 1000 + amount_in_after_fee);

        Some(amount_out / 1000)
    }

    /// A swap does not need to directly modify the pool balances, but the executor
    /// should make sure that required funds are provided.
    pub fn swap(&mut self, t_in: Unit, amount_in: u64, swap: SwapArgs) {
        // TODO: calculate amout outside proof and check here for efficiency
        let amount_out = self.amount_out(t_in, swap.output.unit, amount_in).unwrap();

        let pair = Pair::new(t_in, swap.output.unit);
        let pool = self.pools.get_mut(&pair).unwrap();

        let (balance_in, balance_out) = if pair.t0 == t_in {
            (&mut pool.balance_0, &mut pool.balance_1)
        } else {
            (&mut pool.balance_1, &mut pool.balance_0)
        };

        *balance_in += amount_in;
        *balance_out -= amount_out;
        self.swaps_output.push(swap.to_output(amount_out));
    }

    /// Check no pool notes are used in this tx
    pub fn validate_no_pools(&self, zone_update: &LedgerUpdate) -> bool {
        self.nfs.iter().all(|nf| !zone_update.has_input(nf))
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

    pub fn expected_pool_balances(&self) -> BTreeMap<Unit, u64> {
        let mut expected_pool_balances = BTreeMap::new();
        for (Pair { t0, t1 }, pool) in self.pools.iter() {
            *expected_pool_balances.entry(*t0).or_insert(0) += pool.balance_0;
            *expected_pool_balances.entry(*t1).or_insert(0) += pool.balance_1;
        }

        expected_pool_balances
    }

    // TODO: only for testing purposes
    pub fn add_liquidity(&mut self, t0_unit: Unit, t1_unit: Unit, t0_in: u64, t1_in: u64) {
        let pair = Pair::new(t0_unit, t1_unit);
        let pool = self.pools.entry(pair).or_insert(Pool {
            balance_0: 0,
            balance_1: 0,
            shares_unit: get_pair_share_unit(pair).unit(),
            total_shares: 0,
        });
        let (balance_0, balance_1) = if pair.t0 == t0_unit {
            (&mut pool.balance_0, &mut pool.balance_1)
        } else {
            (&mut pool.balance_1, &mut pool.balance_0)
        };

        *balance_0 += t0_in;
        *balance_1 += t1_in;
    }

    pub fn check_swaps_executed(&self, tx: &Tx) {
        let zone_update = tx.updates.get(&self.zone_id).unwrap();
        for output in &self.swaps_output {
            assert!(zone_update.has_output(&output.note_commitment()));
        }
    }

    pub fn update_and_commit(mut self, tx: &Tx, pool_notes: &[InputWitness]) -> [u8; 32] {
        self.pools_update(&tx, pool_notes);
        self.check_swaps_executed(&tx);
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

impl Default for ZoneData {
    fn default() -> Self {
        Self::new()
    }
}
