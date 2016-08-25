const { Api } = window.parity;

const DIVISOR = 10 ** 6;

export function formatBlockNumber (blockNumber) {
  return blockNumber.eq(0)
    ? 'Pending'
    : `#${blockNumber.toFormat()}`;
}

export function formatCoins (amount, decimals = 6) {
  return amount.div(DIVISOR).toFormat(decimals);
}

export function formatEth (eth, decimals = 3) {
  return Api.format.fromWei(eth).toFormat(decimals);
}
