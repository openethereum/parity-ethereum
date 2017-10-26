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

import interfaces from './';
import * as customTypes from './types';

const allowedTypes = [Array, Boolean, Object, String].concat(Object.values(customTypes));

function verifyType (obj) {
  if (typeof obj !== 'string') {
    expect(obj).to.satisfy(() => allowedTypes.includes(obj.type));
  }
}

describe('jsonrpc/interfaces', () => {
  Object.keys(interfaces).forEach((group) => {
    describe(group, () => {
      Object.keys(interfaces[group]).forEach((name) => {
        const method = interfaces[group][name];

        describe(name, () => {
          it('has the correct interface', () => {
            expect(method.desc).to.be.a('string');
            expect(method.params).to.be.an('array');
            expect(method.returns).to.satisfy((returns) => {
              return typeof returns === 'string' || typeof returns === 'object';
            });

            method.params.forEach(verifyType);
            verifyType(method.returns);
          });
        });
      });
    });
  });
});
