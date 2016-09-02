import 'isomorphic-fetch';
import es6Promise from 'es6-promise';

import Api from './api';
import { IdentityIcon } from './ui';

es6Promise.polyfill();

window.parity = {
  Api: Api,
  react: {
    IdentityIcon
  }
};
