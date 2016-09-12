import BigNumber from 'bignumber.js';

let lastBlock = new BigNumber(-1);

export const ethBlockNumber = (api, updateAll) => {
  const nextTimeout = () => setTimeout(() => {
    ethBlockNumber(api, updateAll);
  }, 1000);

  api.eth
    .blockNumber()
    .then((blockNumber) => {
      if (blockNumber.gt(lastBlock)) {
        lastBlock = blockNumber;
        updateAll(null, blockNumber);
      }

      nextTimeout();
    })
    .catch(nextTimeout);
};
