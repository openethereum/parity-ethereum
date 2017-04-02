// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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
import 'mock-local-storage';

import es6Promise from 'es6-promise';
es6Promise.polyfill();

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import chaiEnzyme from 'chai-enzyme';
import 'sinon-as-promised';
import sinonChai from 'sinon-chai';
import { WebSocket } from 'mock-socket';
import jsdom from 'jsdom';

chai.use(chaiAsPromised);
chai.use(chaiEnzyme());
chai.use(sinonChai);

// expose expect to global so we won't have to manually import & define it in every test
global.expect = chai.expect;
global.WebSocket = WebSocket;

// setup jsdom
global.document = jsdom.jsdom('<!doctype html><html><body></body></html>');
global.window = document.defaultView;
global.navigator = global.window.navigator;
global.location = global.window.location;
global.Blob = () => {};

// attach mocked localStorage onto the window as exposed by jsdom
global.window.localStorage = global.localStorage;

module.exports = {};
