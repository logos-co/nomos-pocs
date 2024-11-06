use std::collections::BTreeMap;

use ledger_proof_statements::{
    constraint::ConstraintPublic,
    ptx::{PtxPrivate, PtxPublic},
};

use crate::{
    constraint::ConstraintProof,
    error::{Error, Result},
};

const MAX_NOTE_COMMS: usize = 2usize.pow(8);

pub struct ProvedPartialTx {
    pub ptx: cl::PartialTx,
    pub cm_root: [u8; 32],
    pub constraint_proofs: BTreeMap<cl::Nullifier, ConstraintProof>,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedPartialTx {
    pub fn prove(
        ptx: &cl::PartialTxWitness,
        constraint_proofs: BTreeMap<cl::Nullifier, ConstraintProof>,
        note_commitments: &[cl::NoteCommitment],
    ) -> Result<ProvedPartialTx> {
        let cm_leaves = note_commitment_leaves(note_commitments);

        let input_cm_paths = Vec::from_iter(ptx.inputs.iter().map(|input| {
            let output_cm = input.note_commitment();

            let cm_idx = note_commitments
                .iter()
                .position(|c| c == &output_cm)
                .unwrap();

            cl::merkle::path(cm_leaves, cm_idx)
        }));
        let cm_root = cl::merkle::root(cm_leaves);
        let ptx_private = PtxPrivate {
            ptx: ptx.clone(),
            input_cm_paths,
            cm_root,
        };

        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&ptx_private)
            .unwrap()
            .build()
            .unwrap();

        // Obtain the default prover.
        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        // Proof information by proving the specified ELF binary.
        // This struct contains the receipt along with statistics about execution of the guest
        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, nomos_cl_risc0_proofs::PTX_ELF, &opts)
            .map_err(|_| Error::Risc0ProofFailed)?;

        println!(
            "STARK 'ptx' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        Ok(Self {
            ptx: ptx.commit(),
            cm_root,
            risc0_receipt: prove_info.receipt,
            constraint_proofs,
        })
    }

    pub fn public(&self) -> Result<PtxPublic> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self) -> bool {
        let Ok(proved_ptx_inputs) = self.public() else {
            return false;
        };
        let expected_ptx_inputs = PtxPublic {
            ptx: self.ptx.clone(),
            cm_root: self.cm_root,
        };
        if expected_ptx_inputs != proved_ptx_inputs {
            return false;
        }

        let ptx_root = self.ptx.root();

        for input in self.ptx.inputs.iter() {
            let nf = input.nullifier;
            let Some(constraint_proof) = self.constraint_proofs.get(&nf) else {
                return false;
            };
            if input.constraint != constraint_proof.constraint() {
                // ensure the constraint proof is actually for this input
                return false;
            }

            let proved_public = constraint_proof.public().unwrap();
            // TODO: validator must validate that the proved block height is within the range of the executor ticket
            if !constraint_proof.verify(ConstraintPublic { nf, ptx_root, block_height: proved_public.block_height }) {
                // verify the constraint was satisfied
                return false;
            }
        }

        self.risc0_receipt
            .verify(nomos_cl_risc0_proofs::PTX_ID)
            .is_ok()
    }
}

fn note_commitment_leaves(note_commitments: &[cl::NoteCommitment]) -> [[u8; 32]; MAX_NOTE_COMMS] {
    let note_comm_bytes = Vec::from_iter(note_commitments.iter().map(|c| c.as_bytes().to_vec()));
    cl::merkle::padded_leaves::<MAX_NOTE_COMMS>(&note_comm_bytes)
}
