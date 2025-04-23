import 'dotenv/config.js';

const REQUIRED_ENV = [
    "REACT_APP_NOMISWAP_ROUTER_ADDRESS",
    "REACT_APP_NOMISWAP_TOKEN_MEM",
    "REACT_APP_NOMISWAP_TOKEN_NET"
];

const missingVars = REQUIRED_ENV.filter(key => !process.env[key]);
if (missingVars.length > 0) {
    console.error(`❌ Missing required environment variables: ${missingVars.join(', ')}`);
    process.exit(1);
}

const config = {
    routerAddress: process.env.REACT_APP_NOMISWAP_ROUTER_ADDRESS,
    tokenMemAddress: process.env.REACT_APP_NOMISWAP_TOKEN_MEM,
    tokenNetAddress: process.env.REACT_APP_NOMISWAP_TOKEN_NET
};

export default config;