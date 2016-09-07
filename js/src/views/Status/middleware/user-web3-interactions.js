
import { Web3Base } from '../provider/web3-base';

export default class WebInteractions extends Web3Base {

  toMiddleware () {
    return store => next => action => {
      let delegate;
      if (action.type.indexOf('modify ') > -1) {
        delegate = ::this.onModify;
      } else {
        next(action);
        return;
      }

      if (!delegate) {
        return;
      }

      delegate(store, next, action);
    };
  }

  onModify (store, next, action) {
    this.ethcoreWeb3[this.getMethod(action.type)](action.payload);
    action.type = action.type.replace('modify ', 'update ');
    return next(action);
  }

  getMethod (actionType) {
    let method = actionType.split('modify ')[1];
    return 'set' + method[0].toUpperCase() + method.slice(1);
  }
}
