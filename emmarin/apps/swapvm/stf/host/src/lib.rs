use app::{StateUpdate, ZoneOp};
use cl::mantle::{ledger::Ledger, zone::ZoneData};
use ledger_proof_statements::ledger::SyncLog;
use methods::{STF_ELF, STF_ID};
use risc0_zkvm::{ExecutorEnv, Prover, Receipt, Result};

pub struct StfPrivate {
    pub zone_data: ZoneData,
    pub old_ledger: Ledger,
    pub new_ledger: Ledger,
    pub sync_logs: Vec<SyncLog>,
    pub ops: Vec<ZoneOp>,
    pub update_tx: StateUpdate,
}

impl StfPrivate {
    pub fn prove(&self, prover: &impl Prover) -> Result<Receipt> {
        let env = ExecutorEnv::builder()
            .write(&self.zone_data)?
            .write(&self.old_ledger)?
            .write(&self.new_ledger)?
            .write(&self.sync_logs)?
            .write(&STF_ID)?
            .write(&self.ops)?
            .write(&self.update_tx)?
            .build()?;

        let prove_info = prover.prove(env, STF_ELF)?;

        debug_assert!(prove_info.receipt.verify(STF_ID).is_ok());
        Ok(prove_info.receipt)
    }
}
