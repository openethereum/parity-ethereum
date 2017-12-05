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

import Interface from './interface';
import ParamType from './paramType';
import Token from '../token';

describe('abi/spec/Interface', () => {
  const construct = {
    type: 'constructor',
    inputs: []
  };
  const event = {
    type: 'event',
    name: 'Event2',
    anonymous: false,
    inputs: [{ name: 'a', type: 'uint256', indexed: true }, { name: 'b', type: 'bytes32', indexed: false }]
  };
  const func = {
    type: 'function',
    name: 'foo',
    inputs: [{ name: 'a', type: 'uint256' }],
    outputs: []
  };

  describe('parseABI', () => {
    it('throws on invalid types', () => {
      expect(() => Interface.parseABI([{ type: 'noMatch' }])).to.throw(/noMatch/);
    });

    it('creates constructors', () => {
      expect(Interface.parseABI([ construct ])).to.deep.equal([{ _inputs: [] }]);
    });

    it('creates events', () => {
      expect(Interface.parseABI([ event ])[0].name).to.equal('Event2');
    });

    it('creates functions', () => {
      expect(Interface.parseABI([ func ])[0].name).to.equal('foo');
    });

    it('parse complex interfaces', () => {
      expect(Interface.parseABI([ construct, event, func ]).length).to.equal(3);
    });
  });

  describe('constructor', () => {
    const int = new Interface([ construct, event, func ]);

    it('contains the full interface', () => {
      expect(int.interface.length).to.equal(3);
    });

    it('contains the constructors', () => {
      expect(int.constructors.length).to.equal(1);
    });

    it('contains the events', () => {
      expect(int.events.length).to.equal(1);
    });

    it('contains the functions', () => {
      expect(int.functions.length).to.equal(1);
    });
  });

  describe('encodeTokens', () => {
    const int = new Interface([ construct, event, func ]);

    it('encodes simple types', () => {
      expect(
        int.encodeTokens(
          [new ParamType('bool'), new ParamType('string'), new ParamType('int'), new ParamType('uint')],
          [true, 'gavofyork', -123, 123]
        )
      ).to.deep.equal([
        new Token('bool', true), new Token('string', 'gavofyork'), new Token('int', -123), new Token('uint', 123)
      ]);
    });

    it('encodes array', () => {
      expect(
        int.encodeTokens(
          [new ParamType('array', new ParamType('bool'))],
          [[true, false, true]]
        )
      ).to.deep.equal([
        new Token('array', [
          new Token('bool', true), new Token('bool', false), new Token('bool', true)
        ])
      ]);
    });

    it('encodes simple with array of array', () => {
      expect(
        int.encodeTokens(
          [
            new ParamType('bool'),
            new ParamType('fixedArray', new ParamType('array', new ParamType('uint')), 2)
          ],
          [true, [[0, 1], [2, 3]]]
        )
      ).to.deep.equal([
        new Token('bool', true),
        new Token('fixedArray', [
          new Token('array', [new Token('uint', 0), new Token('uint', 1)]),
          new Token('array', [new Token('uint', 2), new Token('uint', 3)])
        ])
      ]);
    });
  });
});
