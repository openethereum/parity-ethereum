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

import DecodedLogParam from './decodedLogParam';
import ParamType from '../paramType';
import Token from '../../token';

describe('abi/spec/event/DecodedLogParam', () => {
  describe('constructor', () => {
    const pt = new ParamType('bool');
    const tk = new Token('bool');

    it('disallows kind not instanceof ParamType', () => {
      expect(() => new DecodedLogParam('test', 'param')).to.throw(/ParamType/);
    });

    it('disallows token not instanceof Token', () => {
      expect(() => new DecodedLogParam('test', pt, 'token')).to.throw(/Token/);
    });

    it('stores all parameters received', () => {
      const log = new DecodedLogParam('test', pt, tk);

      expect(log.name).to.equal('test');
      expect(log.kind).to.equal(pt);
      expect(log.token).to.equal(tk);
    });
  });
});
