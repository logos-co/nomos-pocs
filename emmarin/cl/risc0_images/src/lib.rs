// TODO: remove hex and lazy_static dependencies

#[cfg(feature = "nomos_mantle_risc0_proofs")]
pub mod nomos_mantle_risc0_proofs {
    pub static STF_NOP_ID: &str = include_str!("STF_NOP_ID");
    #[cfg(feature = "elf")]
    pub static STF_NOP_ELF: &[u8] = include_bytes!("STF_NOP_ELF");
}
#[cfg(feature = "nomos_mantle_bundle_risc0_proof")]
pub mod nomos_mantle_bundle_risc0_proof {
    pub static BUNDLE_ID: &str = include_str!("BUNDLE_ID");
    #[cfg(feature = "elf")]
    pub static BUNDLE_ELF: &[u8] = include_bytes!("BUNDLE_ELF");
}
#[cfg(feature = "nomos_mantle_tx_risc0_proof")]
pub mod nomos_mantle_tx_risc0_proof {
    pub static TX_ID: &str = include_str!("TX_ID");
    #[cfg(feature = "elf")]
    pub static TX_ELF: &[u8] = include_bytes!("TX_ELF");
}
#[cfg(feature = "ledger_validity_proof")]
pub mod ledger_validity_proof {
    pub static LEDGER_ID: &str = include_str!("LEDGER_ID");
    #[cfg(feature = "elf")]
    pub static LEDGER_ELF: &[u8] = include_bytes!("LEDGER_ELF");
}
