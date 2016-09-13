const { personal } = window.parity.api;

export const set = (accounts) => ({ type: 'accounts set', accounts });

export const fetch = () => (dispatch) => {
  Promise.all([
    personal.listAccounts(),
    personal.accountsInfo()
  ])
  .then(([ addresses, infos ]) => {
    const accounts = addresses.reduce((accounts, address) => {
      if (infos[address]) accounts[address] = { ...infos[address], address }
      return accounts
    }, {})
    dispatch(set(accounts))
  })
  .catch((err) => {
    console.error('could not fetch accounts');
    if (err) console.error(err.stack);
  });
};

export const select = (address) => ({ type: 'accounts select', address });
