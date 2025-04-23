#!/bin/bash

# Define colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR=$(dirname $(realpath "${BASH_SOURCE[0]}"))
cd $SCRIPT_DIR

echo -e "${BLUE}[Step]${NC} ${YELLOW}Installing dependencies${NC}"
npm install

echo ""
echo -e "${BLUE}[Step]${NC} ${YELLOW}Deploying contracts to http://localhost:8545${NC}"
node deploy-contracts.js

if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}✓ Uniswap Deployment Successful${NC}"
else
    echo -e "\n${RED}✗ Uniswap Deployment Failed${NC}"
fi