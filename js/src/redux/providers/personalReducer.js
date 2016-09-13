import { handleActions } from 'redux-actions';

const initialState = {
};

export default handleActions({
  personalAccountsInfo (state, action) {
    const { accountsInfo } = action;
    const accounts = {};
    const contacts = {};

    Object.keys(accountsInfo).forEach((address) => {
      const account = accountsInfo[address];
      const { name, meta, uuid } = account;

      if (uuid) {
        accounts[address] = { address, name, meta, uuid };
      } else {
        contacts[address] = { address, name, meta };
      }
    });

    return Object.assign({}, state, {
      accounts,
      hasAccounts: Object.keys(accounts).length !== 0,
      contacts,
      hasContacts: Object.keys(contacts).length !== 0
    });
  }
}, initialState);
