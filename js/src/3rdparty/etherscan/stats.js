import { call } from './call';

function _call (action, test) {
  return call('stats', action, null, test);
}

function price (test = false) {
  return _call('ethprice', test);
}

function supply (test = false) {
  return _call('ethsupply', test);
}

const stats = {
  price: price,
  supply: supply
};

export { stats };
