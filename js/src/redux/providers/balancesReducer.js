function getBalances (state, action) {
  const { balances } = action;

  return Object.assign({}, state, { balances });
}

function getTokens (state, action) {
  const { tokens } = action;

  return Object.assign({}, state, { tokens });
}

export default function balancesReducer (state = {}, action) {
  switch (action.type) {
    case 'getBalances':
      return getBalances(state, action);

    case 'getTokens':
      return getTokens(state, action);

    default:
      return state;
  }
}
