import 'isomorphic-fetch';
import es6Promise from 'es6-promise';

import Api from './api';
import { IdentityIcon } from './ui';

es6Promise.polyfill();

const api = new Api(new Api.Transport.Http('/rpc/'));

window.parity = {
  Api,
  api,
  react: {
    IdentityIcon
  }
};
