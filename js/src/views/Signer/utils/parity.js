import logger from './logger';

import { isExtension } from './extension';

const isProd = process.env.NODE_ENV === 'production';

export const isParityRunning = (path) => {
  const url = isProd || isExtension()
    ? `http://${path}/index.html`
    : `http://${window.location.host}/api/ping`;

  return fetch(url, { method: 'GET' })
    .then(() => true)
    .catch((error) => {
      logger.error('isParityRunning', error);
      return false;
    });
};
