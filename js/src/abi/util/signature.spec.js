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

import { eventSignature, methodSignature } from './signature';

describe('abi/util/signature', () => {
  describe('eventSignature', () => {
    it('encodes signature baz() correctly', () => {
      expect(eventSignature('baz', [])).to.deep.equal({
        id: 'baz()',
        name: 'baz',
        signature: 'a7916fac4f538170f7cd12c148552e2cba9fcd72329a2dd5b07a6fa906488ddf'
      });
    });

    it('encodes signature baz(uint32) correctly', () => {
      expect(eventSignature('baz', [{ type: 'uint', length: 32 }])).to.deep.equal({
        id: 'baz(uint32)',
        name: 'baz',
        signature: '7d68785e8fc871be024b75964bd86d093511d4bc2dc7cf7bea32c48a0efaecb1'
      });
    });

    it('encodes signature baz(uint32, bool) correctly', () => {
      expect(eventSignature('baz', [{ type: 'uint', length: 32 }, { type: 'bool' }])).to.deep.equal({
        id: 'baz(uint32,bool)',
        name: 'baz',
        signature: 'cdcd77c0992ec5bbfc459984220f8c45084cc24d9b6efed1fae540db8de801d2'
      });
    });

    it('encodes no-name signature correctly as ()', () => {
      expect(eventSignature(undefined, [])).to.deep.equal({
        id: '()',
        name: undefined,
        signature: ''
      });
    });

    it('encodes no-params signature correctly as ()', () => {
      expect(eventSignature(undefined, undefined)).to.deep.equal({
        id: '()',
        name: undefined,
        signature: ''
      });
    });
  });

  describe('methodSignature', () => {
    it('encodes signature baz() correctly', () => {
      expect(methodSignature('baz', [])).to.deep.equal({
        id: 'baz()',
        name: 'baz',
        signature: 'a7916fac'
      });
    });

    it('encodes signature baz(uint32) correctly', () => {
      expect(methodSignature('baz', [{ type: 'uint', length: 32 }])).to.deep.equal({
        id: 'baz(uint32)',
        name: 'baz',
        signature: '7d68785e'
      });
    });

    it('encodes signature baz(uint32, bool) correctly', () => {
      expect(methodSignature('baz', [{ type: 'uint', length: 32 }, { type: 'bool' }])).to.deep.equal({
        id: 'baz(uint32,bool)',
        name: 'baz',
        signature: 'cdcd77c0'
      });
    });

    it('encodes signature in name correctly', () => {
      expect(methodSignature('baz(uint32,bool)', [{ type: 'uint', length: 32 }, { type: 'bool' }])).to.deep.equal({
        id: 'baz(uint32,bool)',
        name: 'baz',
        signature: 'cdcd77c0'
      });
    });

    it('encodes no-name signature correctly as ()', () => {
      expect(methodSignature(undefined, [])).to.deep.equal({
        id: '()',
        name: undefined,
        signature: ''
      });
    });

    it('encodes no-params signature correctly as ()', () => {
      expect(methodSignature(undefined, undefined)).to.deep.equal({
        id: '()',
        name: undefined,
        signature: ''
      });
    });
  });
});
