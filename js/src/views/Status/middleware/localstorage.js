
import localStore from 'store';
import { syncRpcStateFromLocalStorage } from '../actions/localstorage';
import rpcMetods from '../data/rpc.json';

export default class localStorageMiddleware {

  toMiddleware () {
    return store => next => action => {
      let delegate;
      switch (action.type) {
        case 'add rpcResponse': delegate = ::this.onAddRpcResponse; break;
        case 'reset rpcPrevCalls': delegate = ::this.onResetRpcCalls; break;
        case 'init app': delegate = ::this.onInitApp; break;
        default:
          next(action);
          return;
      }

      if (!delegate) {
        return;
      }

      delegate(store, next, action);
    };
  }

  onInitApp (store, next, action) {
    const prevCalls = localStore.get('rpcPrevCalls');
    if (!(prevCalls && prevCalls.length)) {
      return next(action);
    }

    store.dispatch(syncRpcStateFromLocalStorage({
      prevCalls: prevCalls,
      callNo: prevCalls.length ? prevCalls[0].callNo + 1 : 1,
      selectedMethod: rpcMetods.methods.find(m => m.name === prevCalls[0].name)
    }));
    return next(action);
  }

  onAddRpcResponse (store, next, action) {
    action.payload.callNo = store.getState().rpc.callNo;
    this.unshift('rpcPrevCalls', action.payload);
    return next(action);
  }

  onResetRpcCalls (store, next, action) {
    localStore.set('rpcPrevCalls', []);
    return next(action);
  }

  // TODO [adgo] 20.04.2016 remove if/when PR is accepted: https://github.com/marcuswestin/store.js/pull/153
  unshift (key, value) {
    const newArr = [value].concat(localStore.get(key) || []);
    localStore.set(key, newArr);
  }

}
