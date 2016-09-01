export default class SignerMiddleware {
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
  }
}
