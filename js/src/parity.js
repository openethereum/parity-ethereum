import 'isomorphic-fetch';
import es6Promise from 'es6-promise';

import Abi from './abi';
import Api from './api';
import JsonRpc from './jsonrpc';

es6Promise.polyfill();

const api = new Api(new Api.Transport.Http('/rpc/'));

window.parity = {
  Abi,
  Api,
  JsonRpc,
  api
};
