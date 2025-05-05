# ZKSync Era-based Validium deployment

The ZKSync Era-based Validium PoC showcased by Nomos during the IFT All-Hands 2025 in Split was deployed using a forked version of the ZKSync Era framework.
The forked repository can be found at the [following link][zksync-era-fork-repo].

## What is ZKSync Era

For all documentation related to ZKSync Era, including the architecture of L2 chains powered by the framework, please refer to the official [ZKSync Era documentation][zksync-era-docs].

## Deploy a ZKSync Era Validium that uses Nomos DA capabilities

This section will guide you through setting up a GPU-proved, ZKSync Era-based L2 that uses Nomos DA capabilities.
If you need to change any of the parameters used here, please follow the relevant ZKSync Era docs for how to do that and their implications with the rest of the options used in this guide.

### Machine setup

Before anything, make sure all the required dependencies are installed by following the [official ZKSync Era guide][zksync-era-dev-setup].

Let's start by cloning the [forked repo][zksync-era-fork-repo] from the `logos-co` organization with `git clone https://github.com/logos-co/zksync-era`.

Then, from the root of the cloned repo, install the forked `zkstack` binary: `cargo install --path ./zkstack_cli/crates/zkstack --locked zkstack --force`.
This will install an updated version of the binary that allows deployers to select Nomos from the available DA options.

### Ecosystem creation and initialization

Next step is to initialize a new ecosystem.
You can create a new ecosystem with the following command from within this folder: `zkstack ecosystem create --ecosystem-name  <ecosystem_name> --l1-network localhost --link-to-code <path_to_forked_repo> --chain-name <chain_name> --chain-id <l2_chain_id> --prover-mode gpu --wallet-creation localhost --l1-batch-commit-data-generator-mode validium --evm-emulator true --start-containers true --update-submodules true --verbose`, and choose `Eth` as the base currency.

Replace the following templates with the actual values you choose: `<ecosystem_name>`, `<path_to_forked_repo>`, `<chain_name>`, `<l2_chain_id>` and refer to the official docs for their meaning.

> **INFO**: In case a real prover is not needed, replace the value of the `--prover-mode gpu` with `--prover-mode local`, according to the official ZKSync Era docs.

Move into the `<ecosystem_name>` folder, then initialize the ecosystem with `zkstack ecosystem init --deploy-erc20 false --deploy-ecosystem true --l1-rpc-url http://127.0.0.1:8545 --deploy-paymaster true --server-db-url postgres://postgres:notsecurepassword@localhost:5432 --server-db-name <db_name> --observability false --update-submodules true --verbose`.

Replace the following templates with the actual values you choose: `<db_name>` and refer to the official docs for their meaning.

When prompted for which DA to use, select `"Nomos"`, and provide the required information, including the URL of a Nomos node, username/password for the HTTP authentication, and a 64-long HEX value (without the leading `0x`) as app ID. e.g., `01ea21912cdcbdd9189d49d07b61543ffdf7064355640eb6cc6fc6d902056d1b`.

### Running the sequencer

Everything is now ready to run the L2 sequencer with `zkstack server --verbose`.
Leave the sequencer running in the background.

You can now connect to the sequencer's RPC interface with any EVM compatible wallet, such as Metamask or Rabby.
The address of the sequencer will by default be `http://localhost:3050`, and the chain ID the one specified with the `<chain_id>` value above.

You can also add the local L1 Ethereum network to the same wallet, which exposes an RPC node at `http://localhost:8545`, if the respective value for it has not been changed in the command listed above.

### Running the prover

In cases where the prover components are required, you need to start the different binaries.
First, set up the prover component with `zkstack prover init --bellman-cuda true --bellman-cuda-dir <bellman_cuda_dir> --setup-compressor-key true --setup-keys true --setup-database true --prover-db-url postgres://postgres:notsecurepassword@localhost:5432 --prover-db-name <prover_db_name> --dont-drop false --use-default false --verbose`.

Replace the following templates with the actual values you choose: `<bellman_cuda_dir>`, `<prover_db_name>`.
This process will take a while since it downloads the required keys for the prover and the compressor.

Start the different prover binaries according to the [ZKSync Era prover docs][zksync-era-prover-docs].

## Example applications

With the L2 running, you can deploy any EVM application.
For instance, we can run the ZKSync Era portal to bridge ETHs from the local L1 to the local L2.

To do this, run the portal with `zkstack portal --verbose`.
Then, via the portal UI, fund the L2 account that has ETHs on the L1: for a local L1 setup, you can use the account `0x36615Cf349d7F6344891B1e7CA7C72883F5dc049` with private key `0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110`.
More accounts are available in the [official ZKSYnc Era documentation][zksync-era-l1-accounts].

[zksync-era-fork-repo]: https://github.com/logos-co/zksync-era
[zksync-era-docs]: https://docs.zksync.io/zk-stack
[zksync-era-dev-setup]: https://github.com/matter-labs/zksync-era/blob/main/docs/src/guides/setup-dev.md
[zksync-era-prover-docs]: https://matter-labs.github.io/zksync-era/prover/latest/03_launch.html
[zksync-era-l1-accounts]: https://docs.zksync.io/zk-stack/running/using-a-local-zk-chain#base-layer-is-the-local-reth-node