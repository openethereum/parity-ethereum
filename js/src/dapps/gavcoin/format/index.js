import BigNumber from 'bignumber.js';

const { api } = window.parity;

const DIVISOR = 10 ** 6;
const ZERO = new BigNumber(0);

export function formatBlockNumber (blockNumber) {
  return ZERO.eq(blockNumber || 0)
    ? 'Pending'
    : `#${blockNumber.toFormat()}`;
}

export function formatCoins (amount, decimals = 6) {
  const adjusted = amount.div(DIVISOR);

  if (decimals === -1) {
    if (adjusted.gte(10000)) {
      decimals = 0;
    } else if (adjusted.gte(1000)) {
      decimals = 1;
    } else if (adjusted.gte(100)) {
      decimals = 2;
    } else if (adjusted.gte(10)) {
      decimals = 3;
    } else {
      decimals = 4;
    }
  }

  return adjusted.toFormat(decimals);
}

export function formatEth (eth, decimals = 3) {
  return api.format.fromWei(eth).toFormat(decimals);
}
