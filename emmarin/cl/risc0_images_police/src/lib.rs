#[cfg(test)]
mod tests {
    fn hash(x: impl AsRef<[u8]>) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        Sha256::digest(x).into()
    }

    #[test]
    fn ensure_images_are_correct() {
        assert_eq!(
            risc0_images::nomos_mantle_risc0_proofs::STF_NOP_ID,
            nomos_mantle_risc0_proofs::STF_NOP_ID,
            "STF_NOP_ID"
        );
        assert_eq!(
            hash(risc0_images::nomos_mantle_risc0_proofs::STF_NOP_ELF),
            hash(nomos_mantle_risc0_proofs::STF_NOP_ELF),
            "STF_NOP_ELF"
        );
        assert_eq!(
            risc0_images::nomos_mantle_bundle_risc0_proof::BUNDLE_ID,
            nomos_mantle_bundle_risc0_proof::BUNDLE_ID,
            "BUNDLE_ID"
        );
        assert_eq!(
            hash(risc0_images::nomos_mantle_bundle_risc0_proof::BUNDLE_ELF),
            hash(nomos_mantle_bundle_risc0_proof::BUNDLE_ELF),
            "BUNDLE_ELF"
        );
        assert_eq!(
            risc0_images::nomos_mantle_tx_risc0_proof::TX_ID,
            nomos_mantle_tx_risc0_proof::TX_ID,
            "TX_ID"
        );
        assert_eq!(
            hash(risc0_images::nomos_mantle_tx_risc0_proof::TX_ELF),
            hash(nomos_mantle_tx_risc0_proof::TX_ELF),
            "TX_ELF"
        );
        assert_eq!(
            risc0_images::ledger_validity_proof::LEDGER_ID,
            ledger_validity_proof::LEDGER_ID,
            "LEDGER_ID"
        );
        assert_eq!(
            hash(risc0_images::ledger_validity_proof::LEDGER_ELF),
            hash(ledger_validity_proof::LEDGER_ELF),
            "LEDGER_ELF"
        );
    }
}
