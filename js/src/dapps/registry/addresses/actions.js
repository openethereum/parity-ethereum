import { personal } from '../parity.js';

export const set = (addresses) => ({ type: 'addresses set', addresses });

export const fetch = () => (dispatch) => {
  Promise.all([ personal.listAccounts(), personal.accountsInfo() ])
  .then(([ accounts, data ]) => {
    const addresses = Object.keys(data)
      .filter((address) => data[address] && !data[address].meta.deleted)
      .map((address) => ({
        ...data[address], address,
        isAccount: accounts.includes(address)
      }))
    dispatch(set(addresses));
  })
  .catch((err) => {
    console.error('could not fetch addresses');
    if (err) console.error(err.stack);
  });
};
