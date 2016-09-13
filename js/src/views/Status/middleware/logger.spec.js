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

import sinon from 'sinon';
import logger from './logger';

describe('MIDDLEWARE: LOGGER', () => {
  describe('MIDDLEWARE', () => {
    const state = { statusLogger: { logging: true } };

    beforeEach('spy console', () => {
      sinon.spy(console, 'log');
      sinon.spy(console, 'error');
    });

    afterEach('unspy console', () => {
      console.log.restore();
      console.error.restore();
    });

    it('should call console.log on non-error msgs', () => {
      // given
      const store = { getState: () => state };
      const next = sinon.spy();
      const action = { type: 'test action' };
      const middleware = logger(store)(next);
      expect(middleware).to.be.a('function');
      expect(action).to.be.an('object');

      // when
      middleware(action);

      // then
      expect(console.error.called).to.be.false;
      expect(console.log.calledOnce).to.be.true;
    });

    it('should call console.log on non-error msgs', () => {
      // given
      const store = { getState: () => state };
      const next = sinon.spy();
      const action = { type: 'test error action' };
      const middleware = logger(store)(next);
      expect(middleware).to.be.a('function');
      expect(action).to.be.an('object');

      // when
      middleware(action);

      // then
      expect(console.log.called).to.be.false;
      expect(console.error.calledOnce).to.be.true;
    });
  });
});
