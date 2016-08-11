import Ws from './ws';

const ws = new Ws('ws://localhost:8546/');

describe('transport/Ws', () => {
  it('connects and makes a call to web3_clientVersion', () => {
    return ws.execute('web3_clientVersion').then((version) => {
      const [client] = version.split('/');

      expect(client === 'Geth' || client === 'Parity').to.be.ok;
    });
  });
});
