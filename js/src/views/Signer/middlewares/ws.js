import logger from '../utils/logger';
import * as actions from '../actions/requests';

export default class LocalstorageMiddleware {

  constructor (ws, setToken) {
    this.setToken = setToken;
    this.ws = ws;
  }

  toMiddleware () {
    return store => next => action => {
      let delegate;
      switch (action.type) {
        case 'update token': delegate = this.onUpdateToken; break;
        case 'start confirmRequest': delegate = this.onConfirmStart; break;
        case 'start rejectRequest': delegate = this.onRejectStart; break;
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

  onUpdateToken = (store, next, action) => {
    this.setToken(action.payload);
    this.ws.init(action.payload);
    next(action);
  }

  onConfirmStart = (store, next, action) => {
    next(action);
    const { id, password } = action.payload;
    // TODO [legacy;todr] Remove
    const method = store.getState().requests.compatibilityMode ? 'personal_confirmTransaction' : 'personal_confirmRequest';

    this.send(method, [ id, {}, password ], (err, txHash) => {
      logger.log('[WS MIDDLEWARE] confirm request cb:', err, txHash);
      if (err || !txHash) {
        store.dispatch(actions.errorConfirmRequest({ id, err: err ? err.message : 'Unable to confirm.' }));
        return;
      }

      store.dispatch(actions.successConfirmRequest({ id, txHash }));
      return;
    });
  }

  onRejectStart = (store, next, action) => {
    next(action);
    const id = action.payload;
    // TODO [legacy;todr] Remove
    const method = store.getState().requests.compatibilityMode ? 'personal_rejectTransaction' : 'personal_rejectRequest';

    this.send(method, [ id ], (err, res) => {
      logger.log('[WS MIDDLEWARE] reject request cb:', err, res);
      if (err) {
        store.dispatch(actions.errorRejectRequest({ id, err: err.message }));
        return;
      }

      store.dispatch(actions.successRejectRequest({ id }));
    });
  }

  send (method, params, callback) {
    const payload = {
      jsonrpc: '2.0',
      method, params
    };
    this.ws.send(payload, callback);
  }

}
