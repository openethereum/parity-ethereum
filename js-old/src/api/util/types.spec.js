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

import { isArray, isError, isFunction, isHex, isInstanceOf, isObject, isString } from './types';
import Eth from '../rpc/eth';

describe('api/util/types', () => {
  describe('isArray', () => {
    it('correctly identifies null as false', () => {
      expect(isArray(null)).to.be.false;
    });

    it('correctly identifies empty array as true', () => {
      expect(isArray([])).to.be.true;
    });

    it('correctly identifies array as true', () => {
      expect(isArray([1, 2, 3])).to.be.true;
    });
  });

  describe('isError', () => {
    it('correctly identifies null as false', () => {
      expect(isError(null)).to.be.false;
    });

    it('correctly identifies Error as true', () => {
      expect(isError(new Error('an error'))).to.be.true;
    });
  });

  describe('isFunction', () => {
    it('correctly identifies null as false', () => {
      expect(isFunction(null)).to.be.false;
    });

    it('correctly identifies function as true', () => {
      expect(isFunction(sinon.stub())).to.be.true;
    });
  });

  describe('isHex', () => {
    it('correctly identifies hex by leading 0x', () => {
      expect(isHex('0x123')).to.be.true;
    });

    it('correctly identifies hex without leading 0x', () => {
      expect(isHex('123')).to.be.true;
    });

    it('correctly identifies non-hex values', () => {
      expect(isHex('123j')).to.be.false;
    });

    it('correctly indentifies non-string values', () => {
      expect(isHex(false)).to.be.false;
      expect(isHex()).to.be.false;
      expect(isHex([1, 2, 3])).to.be.false;
    });
  });

  describe('isInstanceOf', () => {
    it('correctly identifies build-in instanceof', () => {
      expect(isInstanceOf(new String('123'), String)).to.be.true; // eslint-disable-line no-new-wrappers
    });

    it('correctly identifies own instanceof', () => {
      expect(isInstanceOf(new Eth({}), Eth)).to.be.true;
    });

    it('correctly reports false for own', () => {
      expect(isInstanceOf({}, Eth)).to.be.false;
    });
  });

  describe('isObject', () => {
    it('correctly identifies empty object as object', () => {
      expect(isObject({})).to.be.true;
    });

    it('correctly identifies non-empty object as object', () => {
      expect(isObject({ data: '123' })).to.be.true;
    });

    it('correctly identifies Arrays as non-objects', () => {
      expect(isObject([1, 2, 3])).to.be.false;
    });

    it('correctly identifies Strings as non-objects', () => {
      expect(isObject('123')).to.be.false;
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
});
