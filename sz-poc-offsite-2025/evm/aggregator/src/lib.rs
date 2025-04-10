// TODO: The logic to batch multiple of these blocks (or the transactions within them) and send them to DA and generate proofs is still missing. It will have to be added at the offsite.
// This type does not support any recovery mechanism, so if the node is stopped, the state DB should be cleaned before starting again.
#[derive(Default)]
pub struct Aggregator<Block> {
    unprocessed_blocks: Vec<Block>,
}

impl<Block> Aggregator<Block>
where
    Block: reth_ethereum::primitives::Block,
{
    pub fn process_blocks(&mut self, new_blocks: impl Iterator<Item = Block>) {
        self.unprocessed_blocks.extend(new_blocks);
    }
}
