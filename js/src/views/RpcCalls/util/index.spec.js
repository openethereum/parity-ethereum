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

import { toPromise, identity } from './';

describe('views/Status/util', () => {
  describe('toPromise', () => {
    it('rejects on error result', () => {
      const ERROR = new Error();
      const FN = function (callback) {
        callback(ERROR);
      };

      return toPromise(FN).catch(err => {
        expect(err).to.equal(ERROR);
      });
    });

    it('resolves on success result', () => {
      const SUCCESS = 'ok, we are good';
      const FN = function (callback) {
        callback(null, SUCCESS);
      };

      return toPromise(FN).then(success => {
        expect(success).to.equal(SUCCESS);
      });
    });
  });

  describe('identity', () => {
    it('returns the value passed in', () => {
      const TEST = { abc: 'def' };

      expect(identity(TEST)).to.deep.equal(TEST);
    });
  });
});
