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

import 'mock-local-storage';
import chai from 'chai';
import chaiEnzyme from 'chai-enzyme';
import { configure } from 'enzyme';
import Adapter from 'enzyme-adapter-react-15';
import { JSDOM } from 'jsdom';
import { WebSocket } from 'mock-socket';
import injectTapEventPlugin from 'react-tap-event-plugin';
import sinonChai from 'sinon-chai';

injectTapEventPlugin();

// Configure Enzyme
configure({ adapter: new Adapter() });

chai.use(chaiEnzyme());
chai.use(sinonChai);

// expose expect to global so we won't have to manually import & define it in every test
global.expect = chai.expect;
global.WebSocket = WebSocket;

// setup jsdom
const { window } = new JSDOM('<!doctype html><html><body></body></html>');

global.window = window;
global.Blob = () => {};

Object.keys(window).forEach((key) => {
  if (!global[key]) {
    global[key] = window[key];
  }
});

// attach mocked localStorage onto the window as exposed by jsdom
global.window.localStorage = global.localStorage;

module.exports = {};
