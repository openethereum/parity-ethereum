export function getBalances (balances) {
  return {
    type: 'getBalances',
    balances
  };
}

export function getTokens (tokens) {
  return {
    type: 'getTokens',
    tokens
  };
}
