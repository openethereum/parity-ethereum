import axios from 'axios';
import logger from './logger';

import { isExtension } from './extension';

const isProd = process.env.NODE_ENV === 'production';

export const isParityRunning = path => {
  try {
    return axios.get(isProd || isExtension() ? `http://${path}/index.html` : `http://${window.location.host}/api/ping`)
      .then(res => true)
      .catch(err => {
        logger.warn('[UTIL Parity] err', err);
        return false;
      });
  } catch (err) {
    logger.warn('[UTIL Parity] err', err);
    return new Promise((resolve, reject) => resolve(false));
  }
};
