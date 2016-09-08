import { newError } from './actions';

export default class ErrorsMiddleware {
  toMiddleware () {
    return (store) => (next) => (action) => {
      const { meta } = action;

      if (!meta || !meta.error) {
        next(action);
        return;
      }

      next(newError(meta.error));
    };
  }
}

export function withError (formatter, type = 'default') {
  return (message) => {
    return {
      error: {
        message: formatter(message),
        type
      }
    };
  };
}
