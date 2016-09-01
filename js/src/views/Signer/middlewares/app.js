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
    const { isLoading, isNodeRunning } = store.getState().app;

    if (isLoading) {
      hashHistory.push('/loading');
    } else if (!isNodeRunning) {
      hashHistory.push('/offline');
    } else {
      hashHistory.push('/');
    }
  }

}
