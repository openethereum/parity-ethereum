import { createHttpApi } from '../../../test/e2e/ethapi';
import { isHexNumber } from '../../../../test/types';

describe('ethapi.web3', () => {
  const ethapi = createHttpApi();

  describe('clientVersion', () => {
    it('returns the client version', () => {
      return ethapi.web3.clientVersion().then((version) => {
        const [client] = version.split('/');

        expect(client === 'Parity' || client === 'Geth').to.be.ok;
      });
    });
  });

  describe('sha3', () => {
    it('returns a keccak256 sha', () => {
      const sha = '0xa7916fac4f538170f7cd12c148552e2cba9fcd72329a2dd5b07a6fa906488ddf';
      const hexStr = 'baz()'.split('').map((char) => char.charCodeAt(0).toString(16)).join('');

      return ethapi.web3.sha3(`0x${hexStr}`).then((hash) => {
        expect(isHexNumber(hash)).to.be.true;
        expect(hash).to.equal(sha);
      });
    });
  });
});
