import { TEST_HTTP_URL, mockHttp } from '../../../../test/mockRpc';

import Http from '../../transport/http';
import Trace from './trace';

const instance = new Trace(new Http(TEST_HTTP_URL));

describe('api/rpc/Trace', () => {
  let scope;

  describe('block', () => {
    beforeEach(() => {
      scope = mockHttp([{ method: 'trace_block', reply: { result: [] } }]);
    });

    it('assumes latest blockNumber when not specified', () => {
      return instance.block().then(() => {
        expect(scope.body.trace_block.params).to.deep.equal(['latest']);
      });
    });

    it('passed specified blockNumber', () => {
      return instance.block(0x123).then(() => {
        expect(scope.body.trace_block.params).to.deep.equal(['0x123']);
      });
    });
  });
});
