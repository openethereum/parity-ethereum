import logger from '../util/logger';

export default class LoggerMiddleware {

  toMiddleware () {
    return store => next => action => {
      const msg = [`[${this.now()}] action:`, `${action.type};`, 'payload:', action.payload];
      if (action.type.match('error')) {
        logger.warn(...msg);
      } else {
        logger.log(...msg);
      }
      return next(action);
    };
  }

  now () {
    const date = new Date(Date.now());
    const seconds = this.pad(date.getSeconds());
    const minutes = this.pad(date.getMinutes());
    const hour = this.pad(date.getHours());
    return `${hour}::${minutes}::${seconds}`;
  }

  pad (n) {
    return n < 10 ? '0' + n : n;
  }

}
