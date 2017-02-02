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

import { TEST_HTTP_URL, endpointTest } from '../../test/mockRpc';

import util from './util';
import Api from './api';

import ethereumRpc from '../jsonrpc/';

describe('api/Api', () => {
  describe('constructor', () => {
    it('requires defined/non-null transport object', () => {
      expect(() => new Api()).to.throw(/Api needs transport/);
      expect(() => new Api(null)).to.throw(/Api needs transport/);
    });

    it('requires an execute function on the transport object', () => {
      expect(() => new Api({})).to.throw(/Api needs transport/);
      expect(() => new Api({ execute: true })).to.throw(/Api needs transport/);
    });
  });

  describe('interface', () => {
    const api = new Api(new Api.Transport.Http(TEST_HTTP_URL, -1));

    Object.keys(ethereumRpc).sort().forEach((endpoint) => {
      describe(endpoint, () => {
        Object.keys(ethereumRpc[endpoint]).sort().forEach((method) => {
          endpointTest(api, endpoint, method);
        });
      });
    });
  });

  it('exposes util as static property', () => {
    expect(Api.util).to.equal(util);
  });
});
