import { TEST_HTTP_URL, endpointTest } from '../../test/mockRpc';

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
    const api = new Api(new Api.Transport.Http(TEST_HTTP_URL));

    Object.keys(ethereumRpc).sort().forEach((endpoint) => {
      describe(endpoint, () => {
        Object.keys(ethereumRpc[endpoint]).sort().forEach((method) => {
          endpointTest(api, endpoint, method);
        });
      });
    });
  });
});
