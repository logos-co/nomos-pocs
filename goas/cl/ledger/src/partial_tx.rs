use std::collections::BTreeMap;

use proof_statements::death_constraint::DeathConstraintPublic;

use crate::{death_constraint::DeathProof, input::ProvedInput, output::ProvedOutput};

#[derive(Debug, Clone)]
pub struct PartialTxInput {
    pub input: ProvedInput,
    pub death: DeathProof,
}

impl PartialTxInput {
    fn verify(&self, ptx_root: cl::PtxRoot) -> bool {
        let nf = self.input.input.input.nullifier;
        self.input.input.input.death_cm == self.death.death_commitment() // ensure the death proof is actually for this input
            && self.input.verify() // ensure the input proof is valie
            && self.death.verify(DeathConstraintPublic { nf, ptx_root }) // verify the death constraint was satisfied
    }
}

pub struct ProvedPartialTx {
    pub inputs: Vec<PartialTxInput>,
    pub outputs: Vec<ProvedOutput>,
}

impl ProvedPartialTx {
    pub fn prove(
        ptx: &cl::PartialTxWitness,
        mut death_proofs: BTreeMap<cl::Nullifier, DeathProof>,
        note_commitments: &[cl::NoteCommitment],
    ) -> ProvedPartialTx {
        Self {
            inputs: Vec::from_iter(ptx.inputs.iter().map(|i| {
                PartialTxInput {
                    input: ProvedInput::prove(i, note_commitments),
                    death: death_proofs
                        .remove(&i.nullifier())
                        .expect("Input missing death proof"),
                }
            })),
            outputs: Vec::from_iter(ptx.outputs.iter().map(ProvedOutput::prove)),
        }
    }

    pub fn ptx(&self) -> cl::PartialTx {
        cl::PartialTx {
            inputs: Vec::from_iter(self.inputs.iter().map(|i| i.input.input.input)),
            outputs: Vec::from_iter(self.outputs.iter().map(|o| o.output)),
        }
    }

    pub fn verify_inputs(&self) -> bool {
        let ptx_root = self.ptx().root();
        self.inputs.iter().all(|i| i.verify(ptx_root))
    }

    pub fn verify_outputs(&self) -> bool {
        self.outputs.iter().all(|o| o.verify())
    }

    pub fn verify(&self) -> bool {
        self.verify_inputs() && self.verify_outputs()
    }
}
