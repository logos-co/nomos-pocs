pub mod mmr;

use cl::{balance::Unit, Constraint, NoteCommitment};
use ed25519_dalek::{
    ed25519::{signature::SignerMut, SignatureBytes},
    Signature, SigningKey, VerifyingKey, PUBLIC_KEY_LENGTH,
};
use mmr::{MMRProof, MMR};
use once_cell::sync::Lazy;
use rand_core::CryptoRngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};

// state of the zone
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct StateCommitment(pub [u8; 32]);

pub type AccountId = [u8; PUBLIC_KEY_LENGTH];

// PLACEHOLDER: this is probably going to be NMO?
pub static ZONE_CL_FUNDS_UNIT: Lazy<Unit> = Lazy::new(|| cl::note::derive_unit("NMO"));

pub fn new_account(mut rng: impl CryptoRngCore) -> SigningKey {
    SigningKey::generate(&mut rng)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneMetadata {
    pub zone_constraint: Constraint,
    pub funds_constraint: Constraint,
    pub unit: Unit,
}

impl ZoneMetadata {
    pub fn id(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.zone_constraint.0);
        hasher.update(self.funds_constraint.0);
        hasher.update(self.unit);
        hasher.finalize().into()
    }
}

type ExecVK = [u8; PUBLIC_KEY_LENGTH];
type BlockSegmentIdx = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bid {
    pub segment: BlockSegmentIdx,
    pub exec_vk: ExecVK,
    pub fee: u64,
    // pub blinding: [u8;32]
}

impl Bid {
    pub fn commit(&self) -> BlindBid {
        let mut hasher = Sha256::new();
        hasher.update(self.segment.to_le_bytes());
        hasher.update(self.exec_vk);
        hasher.update(self.fee.to_le_bytes());
        let commitment: [u8; 32] = hasher.finalize().into();

        BlindBid {
            exec_vk: self.exec_vk,
            segment: self.segment,
            commitment,
        }
    }

    pub fn to_bytes(&self) -> [u8; 8 + PUBLIC_KEY_LENGTH + 8] {
        let mut bytes = [0u8; 8 + PUBLIC_KEY_LENGTH + 8];
        bytes[..8].copy_from_slice(&self.segment.to_le_bytes());
        bytes[8..8 + PUBLIC_KEY_LENGTH].copy_from_slice(&self.exec_vk);
        bytes[8 + PUBLIC_KEY_LENGTH..].copy_from_slice(&self.fee.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BlindBid {
    pub segment: BlockSegmentIdx,
    pub exec_vk: ExecVK,
    pub commitment: [u8; 32],
}

impl BlindBid {
    pub fn to_bytes(&self) -> [u8; 8 + PUBLIC_KEY_LENGTH + 32] {
        let mut bytes = [0u8; 8 + PUBLIC_KEY_LENGTH + 32];
        bytes[..8].copy_from_slice(&self.segment.to_le_bytes());
        bytes[8..8 + PUBLIC_KEY_LENGTH].copy_from_slice(&self.exec_vk);
        bytes[8 + PUBLIC_KEY_LENGTH..].copy_from_slice(&self.commitment);
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DutchBid {
    pub scd_best: Bid,
    pub best: Bid,
}

impl DutchBid {
    pub fn apply(&mut self, bid: Bid) {
        if bid.fee > self.best.fee {
            self.scd_best = self.best;
            self.best = bid;
        } else if bid.fee > self.scd_best.fee {
            self.scd_best = bid;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Auction {
    pub blind_bids: BTreeMap<BlockSegmentIdx, BTreeSet<BlindBid>>,
    pub revealed_bids: BTreeMap<BlockSegmentIdx, DutchBid>,
}

pub fn segment_to_height(seg: BlockSegmentIdx) -> u64 {
    seg * 10
}

pub fn height_to_segment(height: u64) -> BlockSegmentIdx {
    height / 10
}

pub const REVEAL_END_OFFSET: u64 = 100;
pub const BID_END_OFFSET: u64 = REVEAL_END_OFFSET + 100;
pub const BID_START_OFFSET: u64 = BID_END_OFFSET + 100;

impl Auction {
    pub fn root(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();

        for blind_bids in self.blind_bids.values() {
            for blind_bid in blind_bids.iter() {
                hasher.update(blind_bid.segment.to_le_bytes());
                hasher.update(blind_bid.commitment);
                hasher.update(blind_bid.exec_vk);
            }
        }
        for dutch_bids in self.revealed_bids.values() {
            hasher.update(dutch_bids.scd_best.segment.to_le_bytes());
            hasher.update(dutch_bids.scd_best.exec_vk);
            hasher.update(dutch_bids.scd_best.fee.to_le_bytes());

            hasher.update(dutch_bids.best.segment.to_le_bytes());
            hasher.update(dutch_bids.best.exec_vk);
            hasher.update(dutch_bids.best.fee.to_le_bytes());
        }
        let root: [u8; 32] = hasher.finalize().into();

        root
    }

    pub fn bid(&mut self, current_height: u64, blind_bid: BlindBid) {
        let ticket_start_height = segment_to_height(blind_bid.segment);
        let bid_start_height = ticket_start_height - BID_START_OFFSET;
        let bid_end_height = ticket_start_height - BID_END_OFFSET;

        if current_height < bid_start_height || current_height >= bid_end_height {
            panic!("current height is not within the bidding range");
        }

        self.blind_bids
            .entry(blind_bid.segment)
            .or_default()
            .insert(blind_bid);
    }

    pub fn reveal(&mut self, current_height: u64, bid: Bid) {
        let ticket_start_height = segment_to_height(bid.segment);

        let reveal_end_height = ticket_start_height - REVEAL_END_OFFSET;

        if current_height >= reveal_end_height {
            panic!("attempting to reveal after reveal deadline");
        }

        let blind_bid = bid.commit();

        if !self
            .blind_bids
            .get(&bid.segment)
            .map(|bids| bids.contains(&blind_bid))
            .unwrap_or(false)
        {
            panic!("attempted to reveal a bid without commitment");
        }

        self.revealed_bids
            .entry(bid.segment)
            .or_insert_with(|| DutchBid {
                scd_best: bid,
                best: bid,
            })
            .apply(bid);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateWitness {
    pub balances: BTreeMap<AccountId, u64>,
    pub ticket_auction: Auction,
    pub included_txs: MMR,
    pub zone_metadata: ZoneMetadata,
}

impl StateWitness {
    pub fn commit(&self) -> StateCommitment {
        self.state_roots().commit()
    }

    pub fn state_roots(&self) -> StateRoots {
        StateRoots {
            included_txs: self.included_txs.clone(),
            zone_id: self.zone_metadata.id(),
            balance_root: self.balances_root(),
            auction_root: self.ticket_auction.root(),
        }
    }

    pub fn apply(self, current_height: u64, tx: Tx) -> (Self, IncludedTxWitness) {
        let mut state = match tx {
            Tx::Withdraw(w) => self.withdraw(w),
            Tx::Deposit(d) => self.deposit(d),
            Tx::TicketBlindBid(blind_bid) => {
                let mut state = self.clone();
                state.ticket_auction.bid(current_height, blind_bid);
                state
            }
            Tx::TicketRevealBid(bid) => {
                let mut state = self.clone();
                state.ticket_auction.reveal(current_height, bid);
                state
            }
        };

        let inclusion_proof = state.included_txs.push(&tx.to_bytes());
        let tx_inclusion_proof = IncludedTxWitness {
            tx,
            proof: inclusion_proof,
        };

        (state, tx_inclusion_proof)
    }

    fn withdraw(mut self, w: Withdraw) -> Self {
        let Withdraw { from, amount } = w;

        let from_balance = self.balances.entry(from).or_insert(0);
        *from_balance = from_balance
            .checked_sub(amount)
            .expect("insufficient funds in account");

        self
    }

    fn deposit(mut self, d: Deposit) -> Self {
        let Deposit { to, amount } = d;

        let to_balance = self.balances.entry(to).or_insert(0);
        *to_balance += to_balance
            .checked_add(amount)
            .expect("overflow in account balance");

        self
    }

    pub fn balances_root(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"NOMOS_BALANCES_ROOT");

        for (k, v) in self.balances.iter() {
            hasher.update(k);
            hasher.update(&v.to_le_bytes());
        }

        hasher.finalize().into()
    }

    pub fn total_balance(&self) -> u64 {
        self.balances.values().sum()
    }
}

impl From<StateCommitment> for [u8; 32] {
    fn from(commitment: StateCommitment) -> [u8; 32] {
        commitment.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Withdraw {
    pub from: AccountId,
    pub amount: u64,
}

impl Withdraw {
    pub fn to_bytes(&self) -> [u8; 40] {
        let mut bytes = [0; 40];
        bytes[0..PUBLIC_KEY_LENGTH].copy_from_slice(&self.from);
        bytes[PUBLIC_KEY_LENGTH..PUBLIC_KEY_LENGTH + 8].copy_from_slice(&self.amount.to_le_bytes());
        bytes
    }
}

/// A deposit of funds into the zone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deposit {
    pub to: AccountId,
    pub amount: u64,
}

impl Deposit {
    pub fn to_bytes(&self) -> [u8; 40] {
        let mut bytes = [0; 40];
        bytes[0..PUBLIC_KEY_LENGTH].copy_from_slice(&self.to);
        bytes[PUBLIC_KEY_LENGTH..PUBLIC_KEY_LENGTH + 8].copy_from_slice(&self.amount.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedBoundTx {
    pub bound_tx: BoundTx,
    #[serde(with = "serde_arrays")]
    pub sig: SignatureBytes,
}

impl SignedBoundTx {
    pub fn sign(bound_tx: BoundTx, signing_key: &mut SigningKey) -> Self {
        let msg = bound_tx.to_bytes();
        let sig = signing_key.sign(&msg).to_bytes();

        Self { bound_tx, sig }
    }

    pub fn verify_and_unwrap(&self) -> BoundTx {
        let msg = self.bound_tx.to_bytes();

        let sig = Signature::from_bytes(&self.sig);
        let vk = self.bound_tx.tx.verifying_key();
        vk.verify_strict(&msg, &sig).expect("Invalid tx signature");

        self.bound_tx
    }
}

/// A Tx that is executed in the zone if and only if the bind is
/// present is the same partial transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundTx {
    pub tx: Tx,
    pub bind: NoteCommitment,
}

impl BoundTx {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.tx.to_bytes());
        bytes.extend(self.bind.as_bytes());
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tx {
    Withdraw(Withdraw),
    Deposit(Deposit),
    TicketBlindBid(BlindBid),
    TicketRevealBid(Bid),
}

impl Tx {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Tx::Withdraw(withdraw) => withdraw.to_bytes().to_vec(),
            Tx::Deposit(deposit) => deposit.to_bytes().to_vec(),
            Tx::TicketBlindBid(bid) => bid.to_bytes().to_vec(),
            Tx::TicketRevealBid(bid) => bid.to_bytes().to_vec(),
        }
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        match self {
            Tx::Withdraw(w) => VerifyingKey::from_bytes(&w.from).unwrap(),
            Tx::Deposit(d) => VerifyingKey::from_bytes(&d.to).unwrap(),
            Tx::TicketBlindBid(blind_bid) => VerifyingKey::from_bytes(&blind_bid.exec_vk).unwrap(),
            Tx::TicketRevealBid(bid) => VerifyingKey::from_bytes(&bid.exec_vk).unwrap(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncludedTxWitness {
    pub tx: Tx,
    pub proof: MMRProof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateRoots {
    pub included_txs: MMR,
    pub zone_id: [u8; 32],
    pub balance_root: [u8; 32],
    pub auction_root: [u8; 32],
}

impl StateRoots {
    pub fn verify_tx_inclusion(&self, tx_inclusion: &IncludedTxWitness) -> bool {
        self.included_txs
            .verify_proof(&tx_inclusion.tx.to_bytes(), &tx_inclusion.proof)
    }

    /// Commitment to the state roots
    pub fn commit(&self) -> StateCommitment {
        let mut hasher = Sha256::new();
        hasher.update(self.included_txs.commit());
        hasher.update(self.zone_id);
        hasher.update(self.balance_root);
        hasher.update(self.auction_root);
        StateCommitment(hasher.finalize().into())
    }
}
