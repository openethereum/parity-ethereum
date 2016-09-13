import { statusBlockNumber, statusCollection, statusLogs } from './statusActions';

export default class Status {
  constructor (store, api) {
    this._api = api;
    this._store = store;
  }

  start () {
    this._subscribeBlockNumber();
    this._pollStatus();
    this._pollLogs();
  }

  _subscribeBlockNumber () {
    this._api.subscribe('eth_blockNumber', (error, blockNumber) => {
      if (error) {
        return;
      }

      this._store.dispatch(statusBlockNumber(blockNumber));
    });
  }

  _pollStatus = () => {
    const nextTimeout = (timeout = 1000) => setTimeout(this._pollStatus, timeout);

    Promise
      .all([
        this._api.web3.clientVersion(),
        this._api.eth.coinbase(),
        this._api.ethcore.defaultExtraData(),
        this._api.ethcore.extraData(),
        this._api.ethcore.gasFloorTarget(),
        this._api.eth.hashrate(),
        this._api.ethcore.minGasPrice(),
        this._api.ethcore.netChain(),
        this._api.ethcore.netPeers(),
        this._api.ethcore.netPort(),
        this._api.ethcore.nodeName(),
        this._api.ethcore.rpcSettings(),
        this._api.eth.syncing()
      ])
      .then(([clientVersion, coinbase, defaultExtraData, extraData, gasFloorTarget, hashrate, minGasPrice, netChain, netPeers, netPort, nodeName, rpcSettings, syncing]) => {
        const isTest = netChain === 'morden' || netChain === 'testnet';

        nextTimeout();
        this._store.dispatch(statusCollection({
          clientVersion,
          coinbase,
          defaultExtraData,
          extraData,
          gasFloorTarget,
          hashrate,
          minGasPrice,
          netChain,
          netPeers,
          netPort,
          nodeName,
          rpcSettings,
          syncing,
          isTest
        }));
      })
      .catch((error) => {
        console.error('_pollStatus', error);
        nextTimeout();
      });
  }

  _pollLogs = () => {
    const nextTimeout = (timeout = 1000) => setTimeout(this._pollLogs, timeout);
    const { devLogsEnabled } = this._store.getState().nodeStatus;

    if (!devLogsEnabled) {
      nextTimeout();
      return;
    }

    Promise
      .all([
        this._api.ethcore.devLogs(),
        this._api.ethcore.devLogsLevels()
      ])
      .then(([devLogs, devLogsLevels]) => {
        nextTimeout();
        this._store.dispatch(statusLogs({
          devLogs: devLogs.slice(-1024),
          devLogsLevels
        }));
      })
      .catch((error) => {
        console.error('_pollLogs', error);
        nextTimeout();
      });
  }
}
