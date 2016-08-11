import { createHttpApi } from '../../../test/e2e/ethapi';

describe('ethapi.trace', () => {
  const ethapi = createHttpApi();

  describe('block', () => {
    it('returns the latest block', () => {
      return ethapi.trace.block().then((block) => {
        expect(block).to.be.ok;
      });
    });

    it('returns a specified block', () => {
      return ethapi.trace.block('0x65432').then((block) => {
        expect(block).to.be.ok;
      });
    });
  });
});
