use std::collections::BTreeMap;

use ledger_proof_statements::{
    death_constraint::DeathConstraintPublic,
    ptx::{PtxPrivate, PtxPublic},
};

use crate::{
    death_constraint::DeathProof, error::{Error, Result}
};

const MAX_NOTE_COMMS: usize = 2usize.pow(8);


pub struct ProvedPartialTx {
    pub ptx: cl::PartialTx,
    pub cm_root: [u8;32],
    pub death_proofs: BTreeMap<cl::Nullifier, DeathProof>,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedPartialTx {
    pub fn prove(
        ptx: &cl::PartialTxWitness,
        death_proofs: BTreeMap<cl::Nullifier, DeathProof>,
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
        // extract the receipt.
        let receipt = prove_info.receipt;

        Ok(Self {
            ptx: ptx.commit(),
            cm_root,
            risc0_receipt: receipt,
            death_proofs,
        })
    }

    pub fn public(&self) -> Result<PtxPublic> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self) -> bool {
        let Ok(proved_ptx_inputs) = self.public() else {
            return false;
        };
        if (PtxPublic { ptx: self.ptx.clone(), cm_root: self.cm_root }) != proved_ptx_inputs {
            return false;
        }

        let ptx_root = self.ptx.root();

        for input in self.ptx.inputs.iter() {
            let nf = input.nullifier;
            let Some(death_proof) = self.death_proofs.get(&nf) else {
                return false;
            };
            if input.death_cm != death_proof.death_commitment() {
                // ensure the death proof is actually for this input
                return false;
            }
            if !death_proof.verify(DeathConstraintPublic { nf, ptx_root }) {
                // verify the death constraint was satisfied
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
    let cm_leaves = cl::merkle::padded_leaves::<MAX_NOTE_COMMS>(&note_comm_bytes);
    cm_leaves
}
