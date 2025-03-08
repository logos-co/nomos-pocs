pub static STF_NOP_ID: &str = include_str!("STF_NOP_ID");
#[cfg(feature = "elf")]
pub static STF_NOP_ELF: &[u8] = include_bytes!("STF_NOP_ELF");

pub static TX_ID: &str = include_str!("TX_ID");
#[cfg(feature = "elf")]
pub static TX_ELF: &[u8] = include_bytes!("TX_ELF");

pub static BUNDLE_ID: &str = include_str!("BUNDLE_ID");
#[cfg(feature = "elf")]
pub static BUNDLE_ELF: &[u8] = include_bytes!("BUNDLE_ELF");

pub static LEDGER_ID: &str = include_str!("LEDGER_ID");
#[cfg(feature = "elf")]
pub static LEDGER_ELF: &[u8] = include_bytes!("LEDGER_ELF");
