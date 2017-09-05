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

import JsonRpcBase from './jsonRpcBase';

const base = new JsonRpcBase();

describe('api/transport/JsonRpcBase', () => {
  describe('encode', () => {
    it('encodes the body correctly, incrementing id', () => {
      const id = base.id;
      const bdy = base.encode('someMethod', ['param1', 'param2']);
      const enc = `{"jsonrpc":"2.0","method":"someMethod","params":["param1","param2"],"id":${id}}`;

      expect(bdy).to.equal(enc);
      expect(base.id - id).to.equal(1);
    });
  });

  describe('setDebug', () => {
    it('starts with disabled flag', () => {
      expect(base.isDebug).to.be.false;
    });

    it('true flag switches on', () => {
      base.setDebug(true);
      expect(base.isDebug).to.be.true;
    });

    it('false flag switches off', () => {
      base.setDebug(true);
      expect(base.isDebug).to.be.true;
      base.setDebug(false);
      expect(base.isDebug).to.be.false;
    });

    describe('logging', () => {
      beforeEach(() => {
        sinon.spy(console, 'log');
        sinon.spy(console, 'error');
      });

      afterEach(() => {
        console.log.restore();
        console.error.restore();
      });

      it('does not log errors with flag off', () => {
        base.setDebug(false);
        base.log('error');
        expect(console.log).to.not.be.called;
      });

      it('does not log errors with flag off', () => {
        base.setDebug(false);
        base.error('error');
        expect(console.error).to.not.be.called;
      });

      it('does log errors with flag on', () => {
        base.setDebug(true);
        base.log('error');
        expect(console.log).to.be.called;
      });

      it('does log errors with flag on', () => {
        base.setDebug(true);
        base.error('error');
        expect(console.error).to.be.called;
      });
    });
  });
});
