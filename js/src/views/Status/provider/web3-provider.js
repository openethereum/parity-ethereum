
import { isArray, isObject, isEqual, compact } from 'lodash';
import { isBigNumber } from 'web3/lib/utils/utils';
import { toPromise } from '../util';
import { Web3Base } from './web3-base';
import * as StatusActions from '../actions/status';
import * as MiningActions from '../actions/mining';
import * as DebugActions from '../actions/debug';

export class Web3Provider extends Web3Base {

  state = {}

  constructor (web3, ethcoreWeb3, store) {
    super(web3, ethcoreWeb3);
    this.store = store;
    this.delay = 500;
    this.running = false;
    this.tickArr = this.getTickArr();
  }

  onStart () {
    toPromise(this.web3.version.getNode)
      .then(StatusActions.updateVersion)
      .then(::this.store.dispatch)
      .catch(err => {
        console.error(err);
        this.store.dispatch(StatusActions.error(err));
      });
  }

  onTickWhenDisconnected () {
    // When disconnected we are only checking single call.
    // After we connect again - onTick should refresh all other results.
    const call = this.tickArr[0];
    return toPromise(call.method)
      .then(call.actionMaker)
      .then(this.store.dispatch)
      .catch(err => {
        this.store.dispatch(StatusActions.error(err));
      });
  }

  onTick () {
    if (this.store.getState().status.disconnected) {
      return this.onTickWhenDisconnected();
    }

    return Promise.all(this.tickArr.map((obj, idx) => {
      if (!obj.actionMaker) {
        console.error(obj);
        throw new Error(`Missing action creator for no ${idx}`);
      }
      return toPromise(obj.method).then(obj.actionMaker)
        .catch(err => {
          const action = obj.actionMaker();
          console.error(`err for ${action.type} with payload ${action.payload}`);
          this.store.dispatch(StatusActions.error(err));
          return false; // don't process errors in the promise chain
        });
    }))
    .then(compact)
    .then(::this.filterChanged)
    .then(::this.updateState)
    .then(actions => actions.map(this.store.dispatch))
    .catch(err => {
      console.error(err);
      this.store.dispatch(StatusActions.error(err));
    });
  }

  getTickArr () {
    return [
      { method: this.web3.eth.getBlockNumber, actionMaker: StatusActions.updateBlockNumber },
      { method: this.web3.eth.getHashrate, actionMaker: StatusActions.updateHashrate },
      { method: this.web3.eth.getAccounts, actionMaker: StatusActions.updateAccounts },
      { method: this.web3.eth.getCoinbase, actionMaker: MiningActions.updateAuthor },
      { method: this.ethcoreWeb3.getMinGasPrice, actionMaker: MiningActions.updateMinGasPrice },
      { method: this.ethcoreWeb3.getGasFloorTarget, actionMaker: MiningActions.updateGasFloorTarget },
      { method: this.ethcoreWeb3.getExtraData, actionMaker: MiningActions.updateExtraData },
      { method: this.ethcoreWeb3.getDefaultExtraData, actionMaker: MiningActions.updateDefaultExtraData },
      { method: this.ethcoreWeb3.getDevLogsLevels, actionMaker: DebugActions.updateDevLogsLevels },
      { method: this.ethcoreWeb3.getDevLogs, actionMaker: DebugActions.updateDevLogs },
      { method: this.ethcoreWeb3.getNetChain, actionMaker: StatusActions.updateNetChain },
      { method: this.ethcoreWeb3.getNetPort, actionMaker: StatusActions.updateNetPort },
      { method: this.ethcoreWeb3.getNetPeers, actionMaker: StatusActions.updateNetPeers },
      { method: this.ethcoreWeb3.getRpcSettings, actionMaker: StatusActions.updateRpcSettings },
      { method: this.ethcoreWeb3.getNodeName, actionMaker: StatusActions.updateNodeName }
    ];
  }

  nextDelay () {
    let noOfErrors = this.store.getState().status.noOfErrors;
    if (noOfErrors === 0) {
      return this.delay;
    }
    return this.delay * (1 + Math.log(noOfErrors));
  }

  start () {
    this.running = true;
    this.onStart();
    this.refreshTick();
    return () => { this.running = false; };
  }

  refreshTick () {
    if (!this.running) {
      return;
    }
    this.onTick().then(() => {
      setTimeout(::this.refreshTick, this.nextDelay());
    });
  }

  filterChanged (actions) {
    return actions.filter(action => {
      const val = this.state[this.actionToStateProp(action)];

      if (isBigNumber(val)) {
        return !val.equals(action.payload);
      }

      if (isArray(val) || isObject(val)) {
        return !isEqual(val, action.payload);
      }

      return val !== action.payload;
    });
  }

  updateState (actions) {
    return actions.map(action => {
      this.state[this.actionToStateProp(action)] = action.payload;
      return action;
    });
  }

  actionToStateProp (action) {
    return action.type.split(' ')[1];
  }

}
