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

import 'babel-polyfill/dist/polyfill.js';
import es6Promise from 'es6-promise';
es6Promise.polyfill();

try {
  if (typeof self.window !== 'undefined') {
    self.window.fetch = require('isomorphic-fetch');
  }
} catch (e) {}

try {
  if (typeof global !== 'undefined') {
    global.fetch = require('node-fetch');
  }
} catch (e) {}

import Api from './api';
import './dev.parity.html';

// commonjs
module.exports = { Api };
// es6 default export compatibility
module.exports.default = module.exports;

if (typeof self !== 'undefined' && typeof self.window !== 'undefined') {
  const api = new Api(new Api.Transport.Http('/rpc/'));

  self.window.parity = {
    Api,
    api
  };
}
