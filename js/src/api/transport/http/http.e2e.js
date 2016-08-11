import Http from './http';

const http = new Http('http://localhost:8545');

describe('transport/Http', () => {
  it('connects and makes a call to web3_clientVersion', () => {
    return http.execute('web3_clientVersion').then((version) => {
      const [client] = version.split('/');

      expect(client === 'Geth' || client === 'Parity').to.be.ok;
    });
  });
});
