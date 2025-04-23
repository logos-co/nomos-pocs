const Web3 = require("web3");
const Factory = require("./node_modules/@uniswap/v2-core/build/UniswapV2Factory.json");
const Router = require("./node_modules/@uniswap/v2-periphery/build/UniswapV2Router02.json");
const ERC20 = require("./node_modules/@openzeppelin/contracts/build/contracts/ERC20PresetFixedSupply.json");
const Pair = require("./node_modules/@uniswap/v2-core/build/UniswapV2Pair.json");
const WETH = require("./node_modules/canonical-weth/build/contracts/WETH9.json");

const RPC = "http://localhost:8545";
const prvKey = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

const GasPrice = 0.000005;
const GasLimit = 6000000;

// deploy Weth
async function deployWeth(web3, sender) {
  try {
    let weth = new web3.eth.Contract(WETH.abi);
    weth = await weth
      .deploy({ data: WETH.bytecode })
      .send({ from: sender, gas: GasLimit, gasprice: GasPrice })

    console.log("Weth address:", weth.options.address);

    return weth.options.address;
  } catch (error) {
    console.log('Weth deployment went wrong! Lets see what happened...')
    console.log(error)
  }
}

// deploy two ERC20 contracts
async function deployTokens(web3, sender) {
  try {
    let tokenMem = new web3.eth.Contract(ERC20.abi);
    let tokenNet = new web3.eth.Contract(ERC20.abi);

    tokenMem = await tokenMem
      .deploy({
        data: ERC20.bytecode,
        arguments: [
          "Mehmet",
          "MEM",
          // 18,
          web3.utils.toWei("9999999999999999999", "ether"),
          sender,
        ],
      })
      .send({ from: sender, gas: GasLimit, gasprice: GasPrice });

    console.log("MEM Token address:", tokenMem.options.address);

    tokenNet = await tokenNet
      .deploy({
        data: ERC20.bytecode,
        arguments: [
          "New Ether",
          "NET",
          // 18,
          web3.utils.toWei("9999999999999999999", "ether"),
          sender,
        ],
      })
      .send({ from: sender, gas: GasLimit, gasprice: GasPrice });

    console.log("NET Token address:", tokenNet.options.address);

    return [tokenMem.options.address, tokenNet.options.address];
  } catch (error) {
    console.log('ERC20 deployment went wrong! Lets see what happened...')
    console.log(error)
  }

}

// deploy a uniswapV2Router
async function deployRouter(web3, factoryAddress, wethAddress, sender) {
  try {
    let router = new web3.eth.Contract(Router.abi);
    router = await router
      .deploy({ data: Router.bytecode, arguments: [factoryAddress, wethAddress] })
      .send({ from: sender, gas: GasLimit, gasprice: GasPrice });

    console.log("Router address:", router.options.address);

    return router.options.address;
  } catch (error) {
    console.log('Router deployment went wrong! Lets see what happened...')
    console.log(error)
  }

}

// deploy a uniswapV2Factory
async function deployFactory(web3, feeToSetter, sender) {
  try {
    let factory = new web3.eth.Contract(Factory.abi);
    factory = await factory
      .deploy({ data: Factory.bytecode, arguments: [feeToSetter] })
      .send({ from: sender, gas: GasLimit, gasprice: GasPrice });

    console.log("Factory address:", factory.options.address);

    return factory.options.address;
  } catch (error) {
    console.log('Factory deployment went wrong! Lets see what happened...')
    console.log(error)
  }

}

async function approve(tokenContract, spender, amount, sender) {
  try {
    await tokenContract.methods
      .approve(spender, amount)
      .send({ from: sender, gas: GasLimit, gasprice: GasPrice })
      .on("transactionHash", function (hash) {
        console.log("transaction hash", hash);
      })
      .on("receipt", function (receipt) {
        console.log("receipt", receipt);
      });
  } catch (err) {
    console.log("the approve transaction reverted! Lets see why...");

    await tokenContract.methods
      .approve(spender, amount)
      .call({ from: sender, gas: GasLimit, gasprice: GasPrice });
  }
}

// check some stuff on a deployed uniswapV2Pair
async function checkPair(
  web3,
  factoryContract,
  tokenMemAddress,
  tokenNetAddress,
  sender,
  routerAddress
) {
  try {
    console.log("tokenMemAddress: ", tokenMemAddress);
    console.log("tokenNetAddress: ", tokenNetAddress);

    const pairAddress = await factoryContract.methods
      .getPair(tokenMemAddress, tokenNetAddress)
      .call();

    console.log("tokenMem Address", tokenMemAddress);
    console.log("tokenNet Address", tokenNetAddress);
    console.log("pairAddress", pairAddress);
    console.log("router address", routerAddress);

    const pair = new web3.eth.Contract(Pair.abi, pairAddress);

    const reserves = await pair.methods.getReserves().call();

    console.log("reserves for tokenMem", web3.utils.fromWei(reserves._reserve0));
    console.log("reserves for tokenNet", web3.utils.fromWei(reserves._reserve1));
  } catch (err) {
    console.log("the check pair reverted! Lets see why...");
    console.log(err);
  }
}

async function deployUniswap() {
  const web3 = new Web3(RPC);
  const account = web3.eth.accounts.wallet.add(prvKey);
  const myAddress = web3.utils.toChecksumAddress(account.address);

  const wethAddress = await deployWeth(web3, myAddress);
  const weth = new web3.eth.Contract(WETH.abi, wethAddress);

  const factoryAddress = await deployFactory(web3, myAddress, myAddress);
  const factory = new web3.eth.Contract(Factory.abi, factoryAddress);

  const routerAddress = await deployRouter(
    web3,
    factoryAddress,
    wethAddress,
    myAddress
  );
  const router = new web3.eth.Contract(Router.abi, routerAddress);

  // const multicallAddress = await deployMulticall(web3, myAddress);
  // const multicall = new web3.eth.Contract(Multicall.abi, multicallAddress);
  const [tokenMemAddress, tokenNetAddress] = await deployTokens(web3, myAddress);

  const tokenMem = new web3.eth.Contract(ERC20.abi, tokenMemAddress);
  const tokenNet = new web3.eth.Contract(ERC20.abi, tokenNetAddress);

  return (tokenMem, tokenMemAddress, tokenNet, tokenNetAddress, myAddress, web3, router, routerAddress, factory, weth, wethAddress)
}

async function addLiquidity(tokenA, tokenAAddress, tokenB, tokenBAddress, myAddress, web3, router, routerAddress, factory, weth, wethAddress) {
  // liquidity
  const amountADesired = web3.utils.toWei("10000000", "ether");
  const amountBDesired = web3.utils.toWei("10000000", "ether");
  const amountAMin = web3.utils.toWei("0", "ether");
  const amountBMin = web3.utils.toWei("0", "ether");

  // deadline
  var BN = web3.utils.BN;
  const time = Math.floor(Date.now() / 1000) + 200000;
  const deadline = new BN(time);

  // before calling addLiquidity we need to approve the router
  // we need to approve atleast amountADesired and amountBDesired
  const spender = router.options.address;
  const amountA = amountADesired;
  const amountB = amountBDesired;

  await approve(tokenA, spender, amountA, myAddress);
  await approve(tokenB, spender, amountB, myAddress);
  await approve(weth, wethAddress, amountA, myAddress);
  await approve(weth, spender, amountA, myAddress);

  // try to add liquidity to a non-existen pair contract
  try {
    await router.methods
      .addLiquidity(
        tokenAAddress,
        tokenBAddress,
        amountADesired,
        amountBDesired,
        amountAMin,
        amountBMin,
        myAddress,
        deadline
      )
      .send({
        from: myAddress,
        gas: GasLimit,
        gasprice: GasPrice,
      })
      .on("transactionHash", function (hash) {
        console.log("transaction hash", hash);
      })
      .on("receipt", function (receipt) {
        console.log("receipt", receipt);
      });
  } catch (err) {
    console.log("the addLiquidity transaction reverted! Lets see why...");

    await router.methods
      .addLiquidity(
        tokenAAddress,
        tokenBAddress,
        amountADesired,
        amountBDesired,
        amountAMin,
        amountBMin,
        myAddress,
        deadline
      )
      .call({
        from: myAddress,
        gas: GasLimit,
        gasprice: GasPrice,
      });
  }

  await checkPair(
    web3,
    factory,
    tokenAAddress,
    tokenBAddress,
    myAddress,
    routerAddress
  );
}

deployUniswap();
