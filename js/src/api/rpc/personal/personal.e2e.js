import { createHttpApi } from '../../../test/e2e/ethapi';
import { isAddress, isBoolean } from '../../../../test/types';

describe.skip('ethapi.personal', () => {
  const ethapi = createHttpApi();
  const password = 'P@55word';
  let address;

  describe('newAccount', () => {
    it('creates a new account', () => {
      return ethapi.personal.newAccount(password).then((_address) => {
        address = _address;
        expect(isAddress(address)).to.be.ok;
      });
    });
  });

  describe('listAccounts', () => {
    it('has the newly-created account', () => {
      return ethapi.personal.listAccounts(password).then((accounts) => {
        expect(accounts.filter((_address) => _address === address)).to.deep.equal([address]);
        accounts.forEach((account) => {
          expect(isAddress(account)).to.be.true;
        });
      });
    });
  });

  describe('unlockAccount', () => {
    it('unlocks the newly-created account', () => {
      return ethapi.personal.unlockAccount(address, password).then((result) => {
        expect(isBoolean(result)).to.be.true;
        expect(result).to.be.true;
      });
    });
  });
});
