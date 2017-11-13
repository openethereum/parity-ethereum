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

const helpers = require('./helpers.spec.js');

const ShapeShift = require('./');
const initShapeshift = (ShapeShift.default || ShapeShift);

const mockget = helpers.mockget;
const mockpost = helpers.mockpost;

describe('shapeshift/rpc', () => {
  let rpc;
  let shapeshift;

  beforeEach(() => {
    shapeshift = initShapeshift(helpers.APIKEY);
    rpc = shapeshift.getRpc();
  });

  describe('GET', () => {
    const REPLY = { test: 'this is some result' };

    let scope;
    let result;

    beforeEach(() => {
      scope = mockget(shapeshift, [{ path: 'test', reply: REPLY }]);

      return rpc
        .get('test')
        .then((_result) => {
          result = _result;
        });
    });

    it('does GET', () => {
      expect(scope.isDone()).to.be.true;
    });

    it('retrieves the info', () => {
      expect(result).to.deep.equal(REPLY);
    });
  });

  describe('POST', () => {
    const REPLY = { test: 'this is some result' };

    let scope;
    let result;

    beforeEach(() => {
      scope = mockpost(shapeshift, [{ path: 'test', reply: REPLY }]);

      return rpc
        .post('test', { input: 'stuff' })
        .then((_result) => {
          result = _result;
        });
    });

    it('does POST', () => {
      expect(scope.isDone()).to.be.true;
    });

    it('retrieves the info', () => {
      expect(result).to.deep.equal(REPLY);
    });

    it('passes the input object', () => {
      expect(scope.body.test.input).to.equal('stuff');
    });

    it('passes the apikey specified', () => {
      expect(scope.body.test.apiKey).to.equal(helpers.APIKEY);
    });
  });
});
