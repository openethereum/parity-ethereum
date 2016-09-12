import BigNumber from 'bignumber.js';

let lastBlock = new BigNumber(-1);

export const ethBlockNumber = (api, updateAll) => {
  const nextTimeout = () => setTimeout(() => {
    ethBlockNumber(api, updateAll);
  }, 1000);

  api.eth
    .blockNumber()
    .then((blockNumber) => {
      if (!blockNumber.eq(lastBlock)) {
        lastBlock = blockNumber;
        updateAll('eth.blockNumber', null, blockNumber);
      }

      nextTimeout();
    })
    .catch(nextTimeout);
};
