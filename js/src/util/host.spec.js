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

import { createLocation } from './host';

describe('createLocation', () => {
  it('only changes the host with no token', () => {
    expect(
      createLocation('', { protocol: 'http:', port: 3000, hostname: 'localhost' })
    ).to.equal('http://127.0.0.1:3000/#/');
  });

  it('preserves hash when changing the host', () => {
    expect(
      createLocation('', { protocol: 'http:', port: 3000, hostname: 'localhost', hash: '#/accounts' })
    ).to.equal('http://127.0.0.1:3000/#/accounts');
  });

  it('adds the token when required', () => {
    expect(
      createLocation('test', { protocol: 'http:', port: 3000, hostname: 'localhost' })
    ).to.equal('http://127.0.0.1:3000/#/?token=test');
  });

  it('preserves hash when token adjusted', () => {
    expect(
      createLocation('test', { protocol: 'http:', port: 3000, hostname: 'localhost', hash: '#/accounts' })
    ).to.equal('http://127.0.0.1:3000/#/accounts?token=test');
  });

  it('does not override already-passed parameters', () => {
    expect(
      createLocation('test', { protocol: 'http:', port: 3000, hostname: 'localhost', hash: '#/accounts?token=abc' })
    ).to.equal('http://127.0.0.1:3000/#/accounts?token=abc');
  });
});
