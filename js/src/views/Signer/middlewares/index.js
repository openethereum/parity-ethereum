// Middleware classes (except logger)
import Ws from './ws';
import Signer from './signer';
import Toastr from './toastr';
import logger from './logger';

export default function middlewares (ws, setToken) {
  // Middleware instances
  const wsMiddleware = new Ws(ws, setToken);
  const signer = new Signer();
  const toastr = new Toastr();

  return [
    logger,
    wsMiddleware.toMiddleware(),
    toastr.toMiddleware(),
    signer.toMiddleware()
  ];
}
