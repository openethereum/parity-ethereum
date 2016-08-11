import EthApi from '../../src/api/ethApi';

function createApi (transport) {
  if (process.env.DEBUG) {
    transport.setDebug(true);
  }

  return new EthApi(transport);
}

export function createHttpApi () {
  return createApi(new EthApi.Transport.Http('http://localhost:8545'));
}

export function createWsApi () {
  return createApi(new EthApi.Transport.Ws('ws://localhost:8546'));
}
