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

import sinon from 'sinon';

import Logging from './logging';

describe('api/subscriptions/logging', () => {
  let cb;
  let logging;

  beforeEach(() => {
    cb = sinon.stub();
    logging = new Logging(cb);
  });

  describe('constructor', () => {
    it('starts the instance in a started state', () => {
      expect(logging.isStarted).to.be.true;
    });
  });

  describe('send', () => {
    const method = 'method';
    const params = 'params';
    const json = 'json';

    beforeEach(() => {
      Logging.send(method, params, json);
    });

    it('calls the subscription update', () => {
      expect(cb).to.have.been.calledWith('logging', null, { method, params, json });
    });
  });
});
