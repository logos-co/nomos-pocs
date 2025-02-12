use ledger_proof_statements::tx::TxPrivate;

use crate::{
    covenant::{SpendingCovenantProof, SupplyCovenantProof},
    error::{Error, Result},
};
use cl::{
    crust::{Tx, TxWitness, UnitWitness},
    ds::mmr::{MMRProof, MMR},
};

#[derive(Debug, Clone)]
pub struct ProvedTx {
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedTx {
    pub fn prove(
        tx_witness: TxWitness,
        mint_units: Vec<UnitWitness>,
        burn_units: Vec<UnitWitness>,
        spend_units: Vec<UnitWitness>,
        supply_covenant_proofs: Vec<SupplyCovenantProof>,
        spending_covenant_proofs: Vec<SpendingCovenantProof>,
    ) -> Result<ProvedTx> {
        let tx_private = TxPrivate {
            tx: tx_witness,
            mint_units,
            burn_units,
            spend_units,
        };

        let mut env = risc0_zkvm::ExecutorEnv::builder();

        for proof in spending_covenant_proofs {
            env.add_assumption(proof.risc0_receipt);
        }

        for proof in supply_covenant_proofs {
            env.add_assumption(proof.risc0_receipt);
        }

        let env = env.write(&tx_private).unwrap().build().unwrap();

        // Obtain the default prover.
        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        // Proof information by proving the specified ELF binary.
        // This struct contains the receipt along with statistics about execution of the guest
        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, nomos_cl_tx_risc0_proof::TX_ELF, &opts)
            .map_err(|_| Error::Risc0ProofFailed)?;

        println!(
            "STARK 'ptx' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        Ok(Self {
            risc0_receipt: prove_info.receipt,
        })
    }

    pub fn public(&self) -> Tx {
        self.risc0_receipt.journal.decode().unwrap()
    }

    pub fn verify(&self) -> bool {
        self.risc0_receipt
            .verify(nomos_cl_tx_risc0_proof::TX_ID)
            .is_ok()
    }
}
