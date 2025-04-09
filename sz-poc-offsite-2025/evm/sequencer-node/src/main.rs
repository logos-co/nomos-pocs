use futures::TryStreamExt as _;
use reth::cli::Cli;
use reth_ethereum::{
    exex::{ExExContext, ExExEvent, ExExNotification},
    node::{EthereumNode, api::FullNodeComponents},
};
use reth_tracing::tracing::info;

async fn push_block_to_da<Node: FullNodeComponents>(
    mut ctx: ExExContext<Node>,
) -> eyre::Result<()> {
    while let Some(notification) = ctx.notifications.try_next().await? {
        match &notification {
            ExExNotification::ChainCommitted { new } => {
                // TODO: Push range of finalized blocks to DA, and interact with prover to generate a proof over the range.
                info!(committed_chain = ?new.range(), "Received commit");
            }
            ExExNotification::ChainReorged { old, new } => {
                info!(from_chain = ?old.range(), to_chain = ?new.range(), "Received reorg");
            }
            ExExNotification::ChainReverted { old } => {
                info!(reverted_chain = ?old.range(), "Received revert");
            }
        };

        if let Some(committed_chain) = notification.committed_chain() {
            ctx.events
                .send(ExExEvent::FinishedHeight(committed_chain.tip().num_hash()))
                .unwrap();
        }
    }

    Ok(())
}

fn main() -> eyre::Result<()> {
    Cli::parse_args().run(|builder, _| {
        Box::pin(async move {
            let handle = Box::pin(
                builder
                    .node(EthereumNode::default())
                    .install_exex("push-block-to-da", async move |ctx| {
                        Ok(push_block_to_da(ctx))
                    })
                    .launch(),
            )
            .await?;

            handle.wait_for_node_exit().await
        })
    })
}
