// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import web3Formatters from 'web3/lib/web3/formatters.js';
import web3Utils from 'web3/lib/utils/utils.js';
import * as RpcActions from '../actions/rpc';
import { hasErrors, filterErrors, isError } from '../util/error';
import RpcProvider from '../provider/rpc-provider';
const rpcProvider = new RpcProvider(web3Utils, web3Formatters);

export default class RpcMiddleware {

  constructor (request) {
    this._request = request;
  }

  toMiddleware () {
    return store => next => action => {
      if (action.type !== 'fire rpc') {
        return next(action);
      }

      const { method, inputFormatters, outputFormatter, params } = action.payload;
      const formattedParams = rpcProvider.formatParams(params, inputFormatters);

      if (hasErrors(formattedParams)) {
        let errors = filterErrors(formattedParams);
        return store.dispatch(RpcActions.error(errors));
      }

      this._request(
        this.getOptions(method, formattedParams),
        this.responseHandler(store, method, params, outputFormatter)
      );
      return next(action);
    };
  }

  responseHandler (store, method, params, outputFormatter) {
    return (err, response, body) => {
      if (err) {
        return store.dispatch(RpcActions.error(err));
      }

      const formattedResult = rpcProvider.formatResult(body.result, outputFormatter);

      if (isError(formattedResult)) {
        return store.dispatch(RpcActions.error(formattedResult));
      }

      const addRpcResponseAction = RpcActions.addRpcReponse({
        name: method,
        params: params,
        response: formattedResult
      });
      store.dispatch(addRpcResponseAction);
    };
  }

  getOptions (method, params) {
    return {
      url: '/rpc/',
      method: 'POST',
      json: {
        id: 1000,
        method: method,
        jsonrpc: '2.0',
        params: params
      }
    };
  }

}
