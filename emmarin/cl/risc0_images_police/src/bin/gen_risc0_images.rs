use base64::prelude::*;

macro_rules! gen_risc0_image {
    ($module:ident, $id:ident, $elf:ident) => {
        println!("pub mod {} {{", stringify!($module));
        println!(
            "  pub const {}: [u32; 8] = {:?};",
            stringify!($id),
            $module::$id
        );
        println!(
            "  pub const {}: &[u8] = binary_macros::base64!({:?});",
            stringify!($elf),
            BASE64_STANDARD.encode(&$module::$elf)
        );
        println!("}}");
    };
}

fn main() {
    gen_risc0_image!(nomos_mantle_risc0_proofs, STF_NOP_ID, STF_NOP_ELF);
    gen_risc0_image!(nomos_mantle_bundle_risc0_proof, BUNDLE_ID, BUNDLE_ELF);
    gen_risc0_image!(nomos_mantle_tx_risc0_proof, TX_ID, TX_ELF);
    gen_risc0_image!(ledger_validity_proof, LEDGER_ID, LEDGER_ELF);
}
