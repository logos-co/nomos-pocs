use bytes::Bytes;
use cryptarchia_engine::Slot;
use groth16::Fr;
use nomos_core::{
    block::Block,
    crypto::{Digest as _, Hasher},
    header::{ContentId, Header, HeaderId},
    mantle::{
        ledger::Tx as LedgerTx, ops::leader_claim::VoucherCm, MantleTx, Note, SignedMantleTx,
        Transaction as _, Utxo,
    },
    proofs::{
        leader_proof::{Groth16LeaderProof, LeaderPrivate, LeaderPublic},
        zksig::{DummyZkSignature, ZkSignaturePublic},
    },
};
use rand::{Rng as _, SeedableRng as _};
use rand_chacha::ChaCha20Rng;

use crate::deterministic::{create_deterministic_rng, RngPurpose};

pub fn create_block(
    block_index: usize,
    parent_id: HeaderId,
) -> Result<(HeaderId, Bytes), Box<dyn std::error::Error>> {
    let transactions = create_signed_mantle_txs(block_index);

    let slot = Slot::from(block_index as u64);
    let block_root = ContentId::from(calculate_block_root(&transactions));
    let proof = make_test_proof(block_index);

    let header = Header::new(parent_id, block_root, slot, proof);
    let header_id = header.id();

    let block: Block<SignedMantleTx> = Block::new(header, transactions);
    let block_bytes = bincode::serialize(&block)?;

    Ok((header_id, Bytes::from(block_bytes)))
}

#[must_use]
pub fn create_block_data(block_index: usize, target_size: usize) -> Bytes {
    create_simplified_realistic_block_data(block_index, target_size)
}

fn make_test_proof(block_index: usize) -> Groth16LeaderProof {
    let public_inputs = LeaderPublic::new(
        Fr::from(block_index as u64),
        Fr::from(block_index as u64 + 1),
        Fr::from(12345u64),
        block_index as u64,
        1_000_000,
    );

    let note = Note::new(1000, Fr::from(block_index as u64).into());
    let utxo = Utxo {
        tx_hash: Fr::from(block_index as u64).into(),
        output_index: 0,
        note,
    };

    let leader_key_bytes = [block_index as u8; 32];
    let leader_key = ed25519_dalek::VerifyingKey::from_bytes(&leader_key_bytes)
        .unwrap_or_else(|_| ed25519_dalek::VerifyingKey::from_bytes(&[1u8; 32]).unwrap());

    let aged_path = vec![];
    let latest_path = vec![];

    let private = LeaderPrivate::new(
        public_inputs,
        utxo,
        &aged_path,
        &latest_path,
        Fr::from(999u64),
        0,
        &leader_key,
    );

    let voucher_cm = VoucherCm::default();

    Groth16LeaderProof::prove(private, voucher_cm).unwrap_or_else(|_| {
        panic!("Proof generation failed - ensure POL_PROOF_DEV_MODE=true is set");
    })
}

#[must_use]
pub fn create_da_share(block: usize, blob: usize, size: usize) -> Bytes {
    let data_id = (block as u64 * 1000) + blob as u64;
    let mut rng = create_deterministic_rng(RngPurpose::DatasetGeneration, data_id);

    let data: Vec<u8> = std::iter::repeat_with(|| rng.gen()).take(size).collect();

    Bytes::from(data)
}

pub async fn create_commitment(
    block: usize,
    blob: usize,
    size: usize,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let commitment_id = (block as u64 * 1000) + blob as u64;
    let mut rng =
        create_deterministic_rng(RngPurpose::DatasetGeneration, commitment_id + 1_000_000);
    let commitment_data: Vec<u8> = std::iter::repeat_with(|| rng.gen()).take(size).collect();

    Ok(Bytes::from(commitment_data))
}

fn create_simplified_realistic_block_data(block_index: usize, target_size: usize) -> Bytes {
    let mut rng = create_deterministic_rng(RngPurpose::DatasetGeneration, block_index as u64);

    let mut block_data = Vec::with_capacity(target_size);

    block_data.extend_from_slice(&(block_index as u64).to_be_bytes());

    let parent_hash: [u8; 32] = rng.gen();
    block_data.extend_from_slice(&parent_hash);

    let merkle_root: [u8; 32] = rng.gen();
    block_data.extend_from_slice(&merkle_root);

    let timestamp = chrono::Utc::now().timestamp() as u64 + block_index as u64 * 30;
    block_data.extend_from_slice(&timestamp.to_be_bytes());

    while block_data.len() < target_size {
        block_data.push(rng.gen());
    }

    block_data.resize(target_size, 0);
    Bytes::from(block_data)
}

fn create_signed_mantle_txs(block_index: usize) -> Vec<SignedMantleTx> {
    let mut rng = ChaCha20Rng::seed_from_u64(block_index as u64 * 12345);

    let tx_count = std::cmp::min(5 + (block_index % 100), 1024);

    let mut transactions = Vec::with_capacity(tx_count);

    for tx_idx in 0..tx_count {
        let input_utxos = create_input_utxos(&mut rng, tx_idx);
        let input_ids: Vec<_> = input_utxos.iter().map(Utxo::id).collect();

        let output_notes = create_output_notes(&mut rng, tx_idx);

        let ledger_tx = LedgerTx::new(input_ids, output_notes);

        let mantle_tx = MantleTx {
            ops: vec![],
            ledger_tx,
            execution_gas_price: rng.gen::<u64>() % 1_000_000,
            storage_gas_price: rng.gen::<u64>() % 100_000,
        };

        let pks: Vec<Fr> = input_utxos.iter().map(|utxo| utxo.note.pk.into()).collect();
        let msg_hash = mantle_tx.hash().into();
        let ledger_tx_proof = DummyZkSignature::prove(ZkSignaturePublic { pks, msg_hash });

        let ops_proofs = vec![];

        let signed_tx = SignedMantleTx {
            ops_proofs,
            ledger_tx_proof,
            mantle_tx,
        };

        transactions.push(signed_tx);
    }

    transactions
}

fn create_input_utxos(rng: &mut ChaCha20Rng, tx_idx: usize) -> Vec<Utxo> {
    let input_count = 1 + (tx_idx % 3);

    (0..input_count)
        .map(|input_idx| Utxo {
            tx_hash: Fr::from(rng.gen::<u64>()).into(),
            output_index: input_idx,
            note: Note::new(
                rng.gen::<u64>() % 1_000_000,
                Fr::from(rng.gen::<u64>()).into(),
            ),
        })
        .collect()
}

fn create_output_notes(rng: &mut ChaCha20Rng, tx_idx: usize) -> Vec<Note> {
    let output_count = 1 + (tx_idx % 4);

    std::iter::repeat_with(|| {
        Note::new(
            rng.gen::<u64>() % 1_000_000,
            Fr::from(rng.gen::<u64>()).into(),
        )
    })
    .take(output_count)
    .collect()
}

fn calculate_block_root(transactions: &[SignedMantleTx]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(b"BLOCK_ROOT_V1");

    for tx in transactions {
        let tx_hash = tx.mantle_tx.hash();
        hasher.update(tx_hash.as_signing_bytes());
    }

    hasher.finalize().into()
}
