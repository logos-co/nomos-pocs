
# ZKSync Era-based Validium Deployment

This repository documents the deployment of a **ZKSync Era-based Validium proof-of-concept (PoC)** showcased by **Nomos** during **IFT All-Hands 2025** in Split.

It uses a forked version of the ZKSync Era framework. You can find the repository here:  
👉 [logos-co/zksync-era][zksync-era-fork-repo]

---

## What is ZKSync Era?

ZKSync Era is a zero-knowledge rollup framework that enables scalable and secure L2 chains.

For detailed documentation, including architecture and components of the framework, visit:  
📚 [ZKSync Era Documentation][zksync-era-docs]

---

## Deploy a ZKSync Era Validium with Nomos DA

This guide walks you through setting up a **GPU-proved Validium L2** chain using **Nomos DA** and the forked ZKSync Era framework.

If you need to modify any parameters, refer to the official ZKSync documentation to understand their implications.

---

### Machine Setup

1. Follow the official setup guide to install dependencies:  
   [ZKSync Era Dev Setup Guide][zksync-era-dev-setup]

2. Clone the forked repo:

   ```bash
   git clone https://github.com/logos-co/zksync-era
   cd zksync-era
   ```

3. Install the forked `zkstack` CLI:

   ```bash
   cargo install --path ./zkstack_cli/crates/zkstack --locked zkstack --force
   ```

---

### Ecosystem Creation

Create a new ecosystem using the `zkstack` CLI:

```bash
zkstack ecosystem create \
  --ecosystem-name <ecosystem_name> \
  --l1-network localhost \
  --link-to-code <path_to_forked_repo> \
  --chain-name <chain_name> \
  --chain-id <l2_chain_id> \
  --prover-mode gpu \
  --wallet-creation localhost \
  --l1-batch-commit-data-generator-mode validium \
  --evm-emulator true \
  --start-containers true \
  --update-submodules true \
  --verbose
```

- Choose `Eth` as the base currency when prompted.
- Replace placeholders with your chosen values.

> ℹ️ For testing without a real GPU prover, use `--prover-mode no-proofs`.

---

### Ecosystem Initialization

1. Navigate into your ecosystem folder:

   ```bash
   cd <ecosystem_name>
   ```

2. Initialize the ecosystem:

   ```bash
   zkstack ecosystem init \
     --deploy-erc20 false \
     --deploy-ecosystem true \
     --l1-rpc-url http://127.0.0.1:8545 \
     --deploy-paymaster true \
     --server-db-url postgres://postgres:notsecurepassword@localhost:5432 \
     --server-db-name <db_name> \
     --observability false \
     --update-submodules true \
     --verbose
   ```

3. When prompted, choose `"Nomos"` as the DA provider and enter:
   - Nomos DA disperal endpoint URL
   - Username/password for HTTP basic auth according to the node configuration
   - A 64-character hex App ID (without the `0x`), e.g.:

     ```
     00ea21912cdcbdd9189d49d07b61543ffdf7064355640eb6cc6fc6d902056d1b
     ```

---

### Running the Sequencer

Start the L2 sequencer:

```bash
zkstack server --verbose
```

- The default RPC endpoint is: `http://localhost:3050`
- Connect with any EVM-compatible wallet (e.g. Metamask, Rabby)
- Use the `chain_id` you configured earlier

Optional: also connect to the local L1 node at `http://localhost:8545`.

---

### Running the Prover

If your use case requires generating ZK proofs:

1. Initialize the prover:

   ```bash
   zkstack prover init \
     --bellman-cuda true \
     --bellman-cuda-dir <bellman_cuda_dir> \
     --setup-compressor-key true \
     --setup-keys true \
     --setup-database true \
     --prover-db-url postgres://postgres:notsecurepassword@localhost:5432 \
     --prover-db-name <prover_db_name> \
     --dont-drop false \
     --use-default false \
     --verbose
   ```

2. Replace placeholders with your paths and database names.

> This process can take time as it downloads all required proving and compression keys.

3. Start prover binaries as explained in the [ZKSync Era Prover Docs][zksync-era-prover-docs].

> As things stand, proving time takes anywhere between 10 minutes to 1 hour, while proof compression typically takes less than one minute. Keep that in mind when trying the whole end-to-end flow.

---

## Example Applications

Once the L2 is running, you can deploy EVM applications on it.

To bridge ETH from the local L1 to the L2, use the ZKSync Era Portal:

```bash
zkstack portal --verbose
```

Use the portal UI to bridge from an L1 account. For a local setup, you can use:

- Address: `0x36615Cf349d7F6344891B1e7CA7C72883F5dc049`
- Private key: `0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110`

More dev accounts can be found in the [ZKSync Era L1 Accounts][zksync-era-l1-accounts].

---

[zksync-era-fork-repo]: https://github.com/logos-co/zksync-era
[zksync-era-docs]: https://docs.zksync.io/zk-stack
[zksync-era-dev-setup]: https://github.com/matter-labs/zksync-era/blob/main/docs/src/guides/setup-dev.md
[zksync-era-prover-docs]: https://matter-labs.github.io/zksync-era/prover/latest/03_launch.html
[zksync-era-l1-accounts]: https://docs.zksync.io/zk-stack/running/using-a-local-zk-chain#base-layer-is-the-local-reth-node
