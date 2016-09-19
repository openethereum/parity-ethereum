const { IdentityIcon } = window.parity.react;
const newContract = window.parity.api.newContract.bind(window.parity.api);
const { personal, ethcore } = window.parity.api;
const { bytesToHex, sha3, toWei, fromWei } = window.parity.api.util;
const getBlockByNumber = window.parity.api.eth.getBlockByNumber.bind(window.parity.api.eth);

export {
  IdentityIcon,
  personal, ethcore, newContract,
  bytesToHex, sha3, toWei, fromWei,
  getBlockByNumber
};
