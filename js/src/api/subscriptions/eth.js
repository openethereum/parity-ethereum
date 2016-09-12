import BigNumber from 'bignumber.js';

export default class PollEth {
  constructor (api, updateSubscriptions) {
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
          this._updateSubscriptions('eth.blockNumber', null, blockNumber);
        }

        nextTimeout();
      })
      .catch(nextTimeout);
  }
}
