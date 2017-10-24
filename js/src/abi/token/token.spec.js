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

import Token from './token';

describe('abi/token/token', () => {
  describe('validateType', () => {
    it('validates address', () => {
      expect(Token.validateType('address')).to.be.true;
    });

    it('validates fixedArray', () => {
      expect(Token.validateType('fixedArray')).to.be.true;
    });

    it('validates array', () => {
      expect(Token.validateType('array')).to.be.true;
    });

    it('validates fixedBytes', () => {
      expect(Token.validateType('fixedBytes')).to.be.true;
    });

    it('validates bytes', () => {
      expect(Token.validateType('bytes')).to.be.true;
    });

    it('validates bool', () => {
      expect(Token.validateType('bool')).to.be.true;
    });

    it('validates int', () => {
      expect(Token.validateType('int')).to.be.true;
    });

    it('validates uint', () => {
      expect(Token.validateType('uint')).to.be.true;
    });

    it('validates string', () => {
      expect(Token.validateType('string')).to.be.true;
    });

    it('throws an error on invalid types', () => {
      expect(() => Token.validateType('noMatch')).to.throw(/noMatch/);
    });
  });

  describe('constructor', () => {
    it('throws an error on invalid types', () => {
      expect(() => new Token('noMatch', '1')).to.throw(/noMatch/);
    });

    it('sets the type of the object', () => {
      expect((new Token('bool', '1')).type).to.equal('bool');
    });

    it('sets the value of the object', () => {
      expect((new Token('bool', '1')).value).to.equal('1');
    });
  });
});
