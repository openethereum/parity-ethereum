import { TEST_HTTP_URL, mockHttp } from '../../../../test/mockRpc';

import Http from '../../transport/http';
import Personal from './personal';

const instance = new Personal(new Http(TEST_HTTP_URL));

describe('rpc/Personal', () => {
  const account = '0x63cf90d3f0410092fc0fca41846f596223979195';
  const checksum = '0x63Cf90D3f0410092FC0fca41846f596223979195';
  let scope;

  describe('accountsInfo', () => {
    it('retrieves the available account info', () => {
      scope = mockHttp([{ method: 'personal_accountsInfo', reply: {
        result: {
          '0x63cf90d3f0410092fc0fca41846f596223979195': {
            name: 'name', uuid: 'uuid', meta: '{"data":"data"}'
          }
        }
      } }]);

      return instance.accountsInfo().then((result) => {
        expect(result).to.deep.equal({
          '0x63Cf90D3f0410092FC0fca41846f596223979195': {
            name: 'name', uuid: 'uuid', meta: {
              data: 'data'
            }
          }
        });
      });
    });
  });

  describe('listAccounts', () => {
    it('retrieves a list of available accounts', () => {
      scope = mockHttp([{ method: 'personal_listAccounts', reply: { result: [account] } }]);

      return instance.listAccounts().then((result) => {
        expect(result).to.deep.equal([checksum]);
      });
    });

    it('returns an empty list when none available', () => {
      scope = mockHttp([{ method: 'personal_listAccounts', reply: { result: null } }]);

      return instance.listAccounts().then((result) => {
        expect(result).to.deep.equal([]);
      });
    });
  });

  describe('newAccount', () => {
    it('passes the password, returning the address', () => {
      scope = mockHttp([{ method: 'personal_newAccount', reply: { result: account } }]);

      return instance.newAccount('password').then((result) => {
        expect(scope.body.personal_newAccount.params).to.deep.equal(['password']);
        expect(result).to.equal(checksum);
      });
    });
  });

  describe('unlockAccount', () => {
    beforeEach(() => {
      scope = mockHttp([{ method: 'personal_unlockAccount', reply: { result: [] } }]);
    });

    it('passes account, password & duration', () => {
      return instance.unlockAccount(account, 'password', 0xf).then(() => {
        expect(scope.body.personal_unlockAccount.params).to.deep.equal([account, 'password', 15]);
      });
    });

    it('provides a default duration when not specified', () => {
      return instance.unlockAccount(account, 'password').then(() => {
        expect(scope.body.personal_unlockAccount.params).to.deep.equal([account, 'password', 1]);
      });
    });
  });
});
