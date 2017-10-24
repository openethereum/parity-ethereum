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

import { stub } from 'sinon';

export const ADDRESS = '0x1111111111111111111111111111111111111111';

export const pastLogs = [
  { event: 'Foo', type: 'mined', address: ADDRESS, params: {} },
  { event: 'Bar', type: 'mined', address: ADDRESS, params: {} }
];

export const liveLogs = [
  { event: 'Bar', type: 'mined', address: ADDRESS, params: { foo: 'bar' } }
];

export const createApi = () => ({
  eth: {
    newFilter: stub().resolves(123),
    uninstallFilter: stub()
      .rejects(new Error('unknown filter id'))
      .withArgs(123).resolves(null),
    getFilterLogs: stub()
      .rejects(new Error('unknown filter id'))
      .withArgs(123).resolves(pastLogs),
    getFilterChanges: stub()
      .rejects(new Error('unknown filter id'))
      .withArgs(123).resolves(liveLogs)
  }
});

export const createContract = (api) => ({
  api,
  address: ADDRESS,
  instance: {
    Foo: { signature: 'Foo signature' },
    Bar: { signature: 'Bar signature' }
  },
  parseEventLogs: stub().returnsArg(0)
});
