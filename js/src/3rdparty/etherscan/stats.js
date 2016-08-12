import { call } from './call';

function _call (action) {
  return call('stats', action);
}

function price () {
  return _call('ethprice');
}

function supply () {
  return _call('ethsupply');
}

const stats = {
  price: price,
  supply: supply
};

export { stats };
