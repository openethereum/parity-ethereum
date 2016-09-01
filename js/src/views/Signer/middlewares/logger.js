import logger from '../utils/logger';

export default store => next => action => {
  if (store.getState().app.logging) {
    const msg = [`[${now()}] action:`, `${action.type};`, 'payload: ', action.payload];
    // const logMethod = action.type.indexOf('error') > -1 ? 'error' : 'log';
    logger.log(...msg); // todo [adgo] - implement error logs
  }
  return next(action);
};

function now () {
  const date = new Date(Date.now());
  const seconds = date.getSeconds();
  const minutes = date.getMinutes();
  const hour = date.getHours();
  return `${hour}::${minutes}::${seconds}`;
}
