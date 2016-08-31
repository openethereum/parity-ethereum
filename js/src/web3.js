import Web3 from 'web3';

import './web3.html';

const http = new Web3.providers.HttpProvider('/rpc/');
const web3 = new Web3(http);

window.web3 = web3;
