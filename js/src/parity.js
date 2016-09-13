import 'isomorphic-fetch';

import es6Promise from 'es6-promise';
es6Promise.polyfill();

import Abi from './abi';
import Api from './api';
import JsonRpc from './jsonrpc';

const api = new Api(new Api.Transport.Http('/rpc/'));

window.parity = {
  Abi,
  Api,
  JsonRpc,
  api
};
