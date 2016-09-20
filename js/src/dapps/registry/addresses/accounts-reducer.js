const initialState = {
  all: {},
  selected: null
};

export default (state = initialState, action) => {
  if (action.type === 'addresses set') {
    const accounts = action.addresses
      .filter((address) => address.isAccount)
      .reduce((accounts, account) => {
        accounts[account.address] = account;
        return accounts;
      }, {});
    return { ...state, all: accounts };
  }

  if (action.type === 'accounts select' && state.all[action.address]) {
    return { ...state, selected: state.all[action.address] };
  }

  return state;
};
