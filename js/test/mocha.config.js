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
