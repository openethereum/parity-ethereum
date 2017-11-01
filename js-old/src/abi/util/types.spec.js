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

import { isArray, isString, isInstanceOf } from './types';
import Token from '../token';

describe('abi/util/types', () => {
  describe('isArray', () => {
    it('correctly identifies empty arrays as Array', () => {
      expect(isArray([])).to.be.true;
    });

    it('correctly identifies non-empty arrays as Array', () => {
      expect(isArray([1, 2, 3])).to.be.true;
    });

    it('correctly identifies strings as non-Array', () => {
      expect(isArray('not an array')).to.be.false;
    });

    it('correctly identifies objects as non-Array', () => {
      expect(isArray({})).to.be.false;
    });
  });

  describe('isString', () => {
    it('correctly identifies empty string as string', () => {
      expect(isString('')).to.be.true;
    });

    it('correctly identifies string as string', () => {
      expect(isString('123')).to.be.true;
    });
  });

  describe('isInstanceOf', () => {
    it('correctly identifies build-in instanceof', () => {
      expect(isInstanceOf(new String('123'), String)).to.be.true; // eslint-disable-line no-new-wrappers
    });

    it('correctly identifies own instanceof', () => {
      expect(isInstanceOf(new Token('int', 123), Token)).to.be.true;
    });

    it('correctly reports false for own', () => {
      expect(isInstanceOf({ type: 'int' }, Token)).to.be.false;
    });
  });
});
