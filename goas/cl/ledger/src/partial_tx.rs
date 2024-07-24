use crate::{input::ProvedInput, output::ProvedOutput};

pub struct ProvedPartialTx {
    pub inputs: Vec<ProvedInput>,
    pub outputs: Vec<ProvedOutput>,
}

impl ProvedPartialTx {
    pub fn prove(
        ptx: &cl::PartialTxWitness,
        note_commitments: &[cl::NoteCommitment],
    ) -> ProvedPartialTx {
        Self {
            inputs: Vec::from_iter(
                ptx.inputs
                    .iter()
                    .map(|i| ProvedInput::prove(i, note_commitments)),
            ),
            outputs: Vec::from_iter(ptx.outputs.iter().map(ProvedOutput::prove)),
        }
    }

    pub fn verify(&self) -> bool {
        self.inputs.iter().all(ProvedInput::verify) && self.outputs.iter().all(ProvedOutput::verify)
    }
}
