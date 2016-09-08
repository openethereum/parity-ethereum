import ErrorsMiddleware, { withError } from './ui/Errors/middleware';

import signerMiddlewares from './views/Signer/middlewares';
import statusMiddlewares from './views/Status/middleware';

export default function (signerWs, signerTokenSetter, statusWeb3) {
  const errors = new ErrorsMiddleware();

  const signer = signerMiddlewares(signerWs, signerTokenSetter);
  const status = statusMiddlewares(statusWeb3);

  const middleware = [
    errors.toMiddleware()
  ];

  return middleware.concat(signer).concat(status);
}

export {
  withError
};
