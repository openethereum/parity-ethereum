import { newError } from './actions';

function withError (formatter, type = 'default') {
  return (message) => {
    return {
      error: {
        message: formatter(message),
        type
      }
    };
  };
}

export default class ErrorsMiddleware {
  toMiddleware () {
    return (store) => (next) => (action) => {
      const { meta } = action;

      if (meta && meta.error) {
        next(newError(meta.error));
      }

      next(action);
    };
  }
}

export {
  withError
};
