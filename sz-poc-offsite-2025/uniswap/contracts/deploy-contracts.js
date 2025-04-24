const Web3 = require("web3");
const Factory = require("./node_modules/@uniswap/v2-core/build/UniswapV2Factory.json");
const Router = require("./node_modules/@uniswap/v2-periphery/build/UniswapV2Router02.json");
const ERC20 = require("./node_modules/@openzeppelin/contracts/build/contracts/ERC20PresetFixedSupply.json");
const WETH = require("./node_modules/canonical-weth/build/contracts/WETH9.json");

const RPC = "http://127.0.0.1:3050";
const prvKey = "7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110";

const gasPrice = 0.000005;
const gasLimit = 60000000;

// deploy Weth
async function deployWeth(web3, sender) {
  try {
    let weth = new web3.eth.Contract(WETH.abi);
    weth = await weth
      .deploy({ data: WETH.bytecode })
      .send({ from: sender, gas: gasLimit, gasprice: gasPrice })

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
    let tokenNmo = new web3.eth.Contract(ERC20.abi);

    tokenMem = await tokenMem
      .deploy({
        data: ERC20.bytecode,
        arguments: [
          "Mehmet",
          "MEM",
          // 18,
          web3.utils.toWei("1000000", "ether"),
          sender,
        ],
      })
      .send({ from: sender, gas: gasLimit, gasprice: gasPrice });

    console.log("MEM Token address:", tokenMem.options.address);

    tokenNmo = await tokenNmo
      .deploy({
        data: ERC20.bytecode,
        arguments: [
          "Nomos",
          "NMO",
          // 18,
          web3.utils.toWei("1000000", "ether"),
          sender,
        ],
      })
      .send({ from: sender, gas: gasLimit, gasprice: gasPrice });

    console.log("NMO Token address:", tokenNmo.options.address);

    return [tokenMem.options.address, tokenNmo.options.address];
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
      .send({ from: sender, gas: gasLimit, gasprice: gasPrice });

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
      .send({ from: sender, gas: gasLimit, gasprice: gasPrice });

    console.log("Factory address:", factory.options.address);

    return factory.options.address;
  } catch (error) {
    console.log('Factory deployment went wrong! Lets see what happened...')
    console.log(error)
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

  const [tokenMemAddress, tokenNmoAddress] = await deployTokens(web3, myAddress);

  const tokenMem = new web3.eth.Contract(ERC20.abi, tokenMemAddress);
  const tokenNmo = new web3.eth.Contract(ERC20.abi, tokenNmoAddress);

  return (tokenMem, tokenMemAddress, tokenNmo, tokenNmoAddress, myAddress, web3, router, routerAddress, factory, weth, wethAddress)
}

deployUniswap();
