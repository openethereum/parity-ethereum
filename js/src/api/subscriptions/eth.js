import BigNumber from 'bignumber.js';

export default class Eth {
  constructor (updateSubscriptions, api) {
    this._api = api;
    this._updateSubscriptions = updateSubscriptions;
    this._started = false;

    this._lastBlock = new BigNumber(-1);
  }

  get isStarted () {
    return this._started;
  }

  start () {
    this._started = true;

    this._blockNumber();
  }

  _blockNumber = () => {
    const nextTimeout = () => setTimeout(this._blockNumber, 1000);

    this._api.eth
      .blockNumber()
      .then((blockNumber) => {
        if (!blockNumber.eq(this._lastBlock)) {
          this._lastBlock = blockNumber;
          this._updateSubscriptions('eth_blockNumber', null, blockNumber);
        }

        nextTimeout();
      })
      .catch(nextTimeout);
  }
}
