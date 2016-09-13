const { IdentityIcon } = window.parity.react;
const newContract = window.parity.api.newContract.bind(window.parity.api);
const { personal, ethcore } = window.parity.api;
const { sha3, toWei, fromWei } = window.parity.api.format;

export {
  IdentityIcon,
  personal, ethcore, newContract,
  sha3, toWei, fromWei
};
