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

import 'isomorphic-fetch';

import es6Promise from 'es6-promise';
es6Promise.polyfill();

import 'mock-local-storage';

import chai from 'chai';
import chaiEnzyme from 'chai-enzyme';
import sinonChai from 'sinon-chai';
import { w3cwebsocket } from 'websocket';
import jsdom from 'jsdom';

chai.use(chaiEnzyme());
chai.use(sinonChai);

// expose expect to global so we won't have to manually import & define it in every test
global.expect = chai.expect;
global.WebSocket = w3cwebsocket;

// setup jsdom
global.document = jsdom.jsdom('<!doctype html><html><body></body></html>');
global.window = document.defaultView;
global.navigator = global.window.navigator;

// attach mocked localStorage onto the window as exposed by jsdom
global.window.localStorage = global.localStorage;

module.exports = {};
