use reth_ethereum::Block;

// TODO: The logic to batch multiple of these blocks (or the transactions within them) and send them to DA and generate proofs is still missing. It will have to be added at the offsite.
// This type does not support any recovery mechanism, so if the node is stopped, the state DB should be cleaned before starting again. The folder is specified by the `--datadir` option in the binary.
#[derive(Default)]
pub struct Aggregator {
    unprocessed_blocks: Vec<Block>,
}

impl Aggregator {
    pub fn process_blocks(&mut self, new_blocks: impl Iterator<Item = Block>) {
        self.unprocessed_blocks.extend(new_blocks);
    }
}
