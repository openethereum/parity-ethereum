'use strict';

const APIKEY = '0x123454321';

const chai = require('chai');
const nock = require('nock');

require('es6-promise').polyfill();
require('isomorphic-fetch');

const shapeshift = require('./index.js')(APIKEY);
const rpc = require('./lib/rpc')(APIKEY);

const mockget = function(requests) {
  let scope = nock(rpc.ENDPOINT);

  requests.forEach((request) => {
    scope = scope
      .get(`/${request.path}`)
      .reply(request.code || 200, () => {
        return request.reply;
      });
  });

  return scope;
};

const mockpost = function(requests) {
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
};

global.expect = chai.expect; // eslint-disable-line no-undef

module.exports = {
  APIKEY: APIKEY,
  mockget: mockget,
  mockpost: mockpost,
  shapeshift: shapeshift,
  rpc: rpc
};
