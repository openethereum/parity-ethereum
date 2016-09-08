export default (store) => {
  return (next) => {
    return (action) => {
      if (store.getState().statusLogger.logging) {
        const msg = [`[${now()}] action:`, `${action.type};`, 'payload: ', action.payload];
        const logMethod = action.type.indexOf('error') > -1 ? 'error' : 'log';

        console[logMethod](...msg);
      }

      return next(action);
    };
  };
};

function now () {
  const date = new Date(Date.now());
  const seconds = date.getSeconds();
  const minutes = date.getMinutes();
  const hour = date.getHours();
  return `${hour}::${minutes}::${seconds}`;
}
