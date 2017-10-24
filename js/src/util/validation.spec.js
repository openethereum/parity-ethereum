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

import BigNumber from 'bignumber.js';

import { NULL_ADDRESS } from './constants';
import { ERRORS, isNullAddress, validateAbi, validateAddress, validateCode, validateName, validatePositiveNumber, validateUint } from './validation';

describe('util/validation', () => {
  describe('validateAbi', () => {
    it('passes on valid ABI', () => {
      const abi = '[{"type":"function","name":"test","inputs":[],"outputs":[]}]';

      expect(validateAbi(abi)).to.deep.equal({
        abi,
        abiError: null,
        abiParsed: [{
          type: 'function',
          name: 'test',
          inputs: [],
          outputs: []
        }],
        error: null
      });
    });

    it('passes on valid ABI & trims ABI', () => {
      const abi = '[ { "type" : "function" , "name" : "test" , "inputs" : [] , "outputs" : [] } ]';

      expect(validateAbi(abi)).to.deep.equal({
        abi: '[{"type":"function","name":"test","inputs":[],"outputs":[]}]',
        abiError: null,
        abiParsed: [{
          type: 'function',
          name: 'test',
          inputs: [],
          outputs: []
        }],
        error: null
      });
    });

    it('sets error on invalid JSON', () => {
      const abi = 'this is not json';

      expect(validateAbi(abi)).to.deep.equal({
        abi,
        abiError: ERRORS.invalidAbi,
        abiParsed: null,
        error: ERRORS.invalidAbi
      });
    });

    it('sets error on non-array JSON', () => {
      const abi = '{}';

      expect(validateAbi(abi)).to.deep.equal({
        abi,
        abiError: ERRORS.invalidAbi,
        abiParsed: {},
        error: ERRORS.invalidAbi
      });
    });

    it('fails with invalid event', () => {
      const abi = '[{ "type":"event" }]';

      expect(validateAbi(abi)).to.deep.equal({
        abi,
        abiError: `${ERRORS.invalidAbi} (#0: event)`,
        abiParsed: [{ type: 'event' }],
        error: `${ERRORS.invalidAbi} (#0: event)`
      });
    });

    it('fails with invalid function', () => {
      const abi = '[{ "type":"function" }]';

      expect(validateAbi(abi)).to.deep.equal({
        abi,
        abiError: `${ERRORS.invalidAbi} (#0: function)`,
        abiParsed: [{ type: 'function' }],
        error: `${ERRORS.invalidAbi} (#0: function)`
      });
    });

    it('fails with unknown type', () => {
      const abi = '[{ "type":"somethingElse" }]';

      expect(validateAbi(abi)).to.deep.equal({
        abi,
        abiError: `${ERRORS.invalidAbi} (#0: somethingElse)`,
        abiParsed: [{ type: 'somethingElse' }],
        error: `${ERRORS.invalidAbi} (#0: somethingElse)`
      });
    });
  });

  describe('validateAddress', () => {
    it('validates address', () => {
      const address = '0x1234567890123456789012345678901234567890';

      expect(validateAddress(address)).to.deep.equal({
        address,
        addressError: null,
        error: null
      });
    });

    it('validates address and converts to checksum', () => {
      const address = '0x5A5eFF38DA95b0D58b6C616f2699168B480953C9';

      expect(validateAddress(address.toLowerCase())).to.deep.equal({
        address,
        addressError: null,
        error: null
      });
    });

    it('sets error on null addresses', () => {
      expect(validateAddress(null)).to.deep.equal({
        address: null,
        addressError: ERRORS.invalidAddress,
        error: ERRORS.invalidAddress
      });
    });

    it('sets error on invalid addresses', () => {
      const address = '0x12344567';

      expect(validateAddress(address)).to.deep.equal({
        address,
        addressError: ERRORS.invalidAddress,
        error: ERRORS.invalidAddress
      });
    });
  });

  describe('validateCode', () => {
    it('validates hex code', () => {
      expect(validateCode('0x123abc')).to.deep.equal({
        code: '0x123abc',
        codeError: null,
        error: null
      });
    });

    it('validates hex code (non-prefix)', () => {
      expect(validateCode('123abc')).to.deep.equal({
        code: '123abc',
        codeError: null,
        error: null
      });
    });

    it('sets error on invalid code', () => {
      expect(validateCode(null)).to.deep.equal({
        code: null,
        codeError: ERRORS.invalidCode,
        error: ERRORS.invalidCode
      });
    });

    it('sets error on empty code', () => {
      expect(validateCode('')).to.deep.equal({
        code: '',
        codeError: ERRORS.invalidCode,
        error: ERRORS.invalidCode
      });
    });

    it('sets error on non-hex code', () => {
      expect(validateCode('123hfg')).to.deep.equal({
        code: '123hfg',
        codeError: ERRORS.invalidCode,
        error: ERRORS.invalidCode
      });
    });
  });

  describe('validateName', () => {
    it('validates names', () => {
      expect(validateName('Joe Bloggs')).to.deep.equal({
        name: 'Joe Bloggs',
        nameError: null,
        error: null
      });
    });

    it('sets error on null names', () => {
      expect(validateName(null)).to.deep.equal({
        name: null,
        nameError: ERRORS.invalidName,
        error: ERRORS.invalidName
      });
    });

    it('sets error on short names', () => {
      expect(validateName('  1  ')).to.deep.equal({
        name: '  1  ',
        nameError: ERRORS.invalidName,
        error: ERRORS.invalidName
      });
    });
  });

  describe('validatePositiveNumber', () => {
    it('validates numbers', () => {
      expect(validatePositiveNumber(123)).to.deep.equal({
        number: 123,
        numberError: null,
        error: null
      });
    });

    it('validates strings', () => {
      expect(validatePositiveNumber('123')).to.deep.equal({
        number: '123',
        numberError: null,
        error: null
      });
    });

    it('validates bignumbers', () => {
      expect(validatePositiveNumber(new BigNumber(123))).to.deep.equal({
        number: new BigNumber(123),
        numberError: null,
        error: null
      });
    });

    it('sets error on invalid numbers', () => {
      expect(validatePositiveNumber(null)).to.deep.equal({
        number: null,
        numberError: ERRORS.invalidAmount,
        error: ERRORS.invalidAmount
      });
    });

    it('sets error on negative numbers', () => {
      expect(validatePositiveNumber(-1)).to.deep.equal({
        number: -1,
        numberError: ERRORS.invalidAmount,
        error: ERRORS.invalidAmount
      });
    });
  });

  describe('validateUint', () => {
    it('validates numbers', () => {
      expect(validateUint(123)).to.deep.equal({
        value: 123,
        valueError: null,
        error: null
      });
    });

    it('validates strings', () => {
      expect(validateUint('123')).to.deep.equal({
        value: '123',
        valueError: null,
        error: null
      });
    });

    it('validates bignumbers', () => {
      expect(validateUint(new BigNumber(123))).to.deep.equal({
        value: new BigNumber(123),
        valueError: null,
        error: null
      });
    });

    it('sets error on invalid numbers', () => {
      expect(validateUint(null)).to.deep.equal({
        value: null,
        valueError: ERRORS.invalidNumber,
        error: ERRORS.invalidNumber
      });
    });

    it('sets error on negative numbers', () => {
      expect(validateUint(-1)).to.deep.equal({
        value: -1,
        valueError: ERRORS.negativeNumber,
        error: ERRORS.negativeNumber
      });
    });

    it('sets error on decimal numbers', () => {
      expect(validateUint(3.1415927)).to.deep.equal({
        value: 3.1415927,
        valueError: ERRORS.decimalNumber,
        error: ERRORS.decimalNumber
      });
    });
  });

  describe('isNullAddress', () => {
    it('verifies a prefixed null address', () => {
      expect(isNullAddress(`0x${NULL_ADDRESS}`)).to.be.true;
    });

    it('verifies a non-prefixed null address', () => {
      expect(isNullAddress(NULL_ADDRESS)).to.be.true;
    });

    it('sets false on a null value', () => {
      expect(isNullAddress(null)).to.be.false;
    });

    it('sets false on a non-full length 00..00 value', () => {
      expect(isNullAddress(NULL_ADDRESS.slice(2))).to.be.false;
    });

    it('sets false on a valid addess, non 00..00 value', () => {
      expect(isNullAddress('0x1234567890123456789012345678901234567890')).to.be.false;
    });
  });
});
