import { createHttpApi } from '../../../test/e2e/ethapi';

describe('ethapi.ethcore', () => {
  const ethapi = createHttpApi();

  describe('gasFloorTarget', () => {
    it('returns and translates the target', () => {
      return ethapi.ethcore.gasFloorTarget().then((value) => {
        expect(value.gt(0)).to.be.true;
      });
    });
  });

  describe('minGasPrice', () => {
    it('returns and translates the price', () => {
      return ethapi.ethcore.minGasPrice().then((value) => {
        expect(value.gt(0)).to.be.true;
      });
    });
  });

  describe('netChain', () => {
    it('returns and the chain', () => {
      return ethapi.ethcore.netChain().then((value) => {
        expect(value).to.equal('morden');
      });
    });
  });

  describe('netMaxPeers', () => {
    it('returns and translates the peers', () => {
      return ethapi.ethcore.netMaxPeers().then((value) => {
        expect(value.gt(0)).to.be.true;
      });
    });
  });

  describe('netPort', () => {
    it('returns and translates the port', () => {
      return ethapi.ethcore.netPort().then((value) => {
        expect(value.gt(0)).to.be.true;
      });
    });
  });

  describe('transactionsLimit', () => {
    it('returns and translates the limit', () => {
      return ethapi.ethcore.transactionsLimit().then((value) => {
        expect(value.gt(0)).to.be.true;
      });
    });
  });

  describe('rpcSettings', () => {
    it('returns and translates the settings', () => {
      return ethapi.ethcore.rpcSettings().then((value) => {
        expect(value).to.be.ok;
      });
    });
  });
});
