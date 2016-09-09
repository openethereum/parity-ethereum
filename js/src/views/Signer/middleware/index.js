// Middleware classes (except logger)
import Ws from './ws';
import Signer from './signer';
import logger from './logger';

export default function middlewares (ws, setToken) {
  // Middleware instances
  const wsMiddleware = new Ws(ws, setToken);
  const signer = new Signer();

  return [
    logger,
    wsMiddleware.toMiddleware(),
    signer.toMiddleware()
  ];
}
