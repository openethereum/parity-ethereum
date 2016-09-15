const { api } = window.parity;

export const SET_ACCOUNTS = 'SET_ACCOUNTS';
export const setAccounts = (accounts) => ({
  type: SET_ACCOUNTS,
  accounts
});

export const SET_SELECTED_ACCOUNT = 'SET_SELECTED_ACCOUNT';
export const setSelectedAccount = (address) => ({
  type: SET_SELECTED_ACCOUNT,
  address
});

export const loadAccounts = () => (dispatch) => {
  Promise
    .all([
      api.personal.listAccounts(),
      api.personal.accountsInfo()
    ])
    .then(results => {
      let [ accounts, accountsInfo ] = results;

      let accountsList = accounts
        .map(address => ({
            ...accountsInfo[address],
            address
        }));

      console.log('accounts', accountsList);

      dispatch(setAccounts(accountsList));
      dispatch(setSelectedAccount(accountsList[0].address));
    })
    .catch(e => {
      console.error('loadAccounts error', e);
    })
};
