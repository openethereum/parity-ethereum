import 'isomorphic-fetch';
import es6Promise from 'es6-promise';

import Api from './api';

es6Promise.polyfill();

const api = new Api(new Api.Transport.Http('/rpc/'));

window.parity = {
  Api,
  api
};
