import { hashHistory } from 'react-router';

export default class AppMiddleware {
  toMiddleware () {
    return store => next => action => {
      next(action);

      // redirect to proper url
      if (action.type === 'update isConnected' || action.type === 'update isNodeRunning') {
        this.redirect(store);
      }
    };
  }

  redirect (store) {
    const { isLoading, isNodeRunning } = store.getState().signer;

    if (isLoading) {
      hashHistory.push('/signer/loading');
    } else if (!isNodeRunning) {
      hashHistory.push('/signer/offline');
    } else {
      hashHistory.push('/signer');
    }
  }

}
