// Middleware classes (except logger)
import Toastr from './toastr';
import logger from './logger';

export default function middlewares (initToken, tokenSetter, wsPath) {
  // Middleware instances
  const toastr = new Toastr();

  return [
    logger,
    toastr.toMiddleware()
  ];
}
