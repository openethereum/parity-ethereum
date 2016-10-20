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

import chai from 'chai';
import nock from 'nock';

global.expect = chai.expect; // eslint-disable-line no-undef

import 'isomorphic-fetch';
import es6Promise from 'es6-promise';
es6Promise.polyfill();

import initShapeshift from './';
import initRpc from './rpc';

const APIKEY = '0x123454321';

const shapeshift = initShapeshift(APIKEY);
const rpc = initRpc(APIKEY);

function mockget (requests) {
  let scope = nock(rpc.ENDPOINT);

  requests.forEach((request) => {
    scope = scope
      .get(`/${request.path}`)
      .reply(request.code || 200, () => {
        return request.reply;
      });
  });

  return scope;
}

function mockpost (requests) {
  let scope = nock(rpc.ENDPOINT);

  requests.forEach((request) => {
    scope = scope
      .post(`/${request.path}`)
      .reply(request.code || 200, (uri, body) => {
        scope.body = scope.body || {};
        scope.body[request.path] = body;

        return request.reply;
      });
  });

  return scope;
}

export {
  APIKEY,
  mockget,
  mockpost,
  shapeshift,
  rpc
};
