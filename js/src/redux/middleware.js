import ErrorsMiddleware from '../ui/Errors/middleware';

import signerMiddleware from '../views/Signer/middleware';
import statusMiddleware from '../views/Status/middleware';

export default function (signerWs, signerTokenSetter, statusWeb3) {
  const errors = new ErrorsMiddleware();

  const signer = signerMiddleware(signerWs, signerTokenSetter);
  const status = statusMiddleware(statusWeb3);

  const middleware = [
    errors.toMiddleware()
  ];

  return middleware.concat(signer).concat(status);
}
