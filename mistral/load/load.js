// load-script-rate.js
import { ethers } from "ethers";

const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

async function main() {
    const provider = new ethers.JsonRpcProvider("http://localhost:8545");
    const sender = ethers.Wallet.fromPhrase(
        "test test test test test test test test test test test junk"
    ).connect(provider);

    const to = "0x1000000000000000000000000000000000000000";
    const value = ethers.parseEther("0.01");
    const maxTxs = 10000;   // total number of txs to send

    console.log(`Sending up to ${maxTxs} txs at ~1 tx/s…`);
    let nonce = await provider.getTransactionCount(sender.address);

    for (let i = 0; i < maxTxs; i++) {
        // fire‐and‐forget each sendTransaction
        sender.sendTransaction({ to, value, gasLimit: 21_000, nonce: nonce + i })
            .then(tx => console.log(`  → tx #${i} sent: ${tx.hash}`))
            .catch(err => console.error(`  ! error on tx #${i}:`, err));

        await sleep(1000);
    }

    console.log("Done scheduling txs.");
}

main().catch(console.error);