import registryAbi from './abi/registry.json';
const { api } = window.parity;
import * as lookup from './Lookup/actions.js';
import * as events from './events/actions.js';

export { lookup, events };

export const setAccounts = (accounts) => ({ type: 'set accounts', accounts });

export const fetchAccounts = () => (dispatch) =>
  Promise.all([
    api.personal.listAccounts(),
    api.personal.accountsInfo()
  ])
  .then(([ addresses, infos ]) => {
    const accounts = addresses.reduce((accounts, address) => {
      if (infos[address]) accounts[address] = { ...infos[address], address }
      return accounts
    }, {})
    dispatch(setAccounts(accounts))
  })
  .catch((err) => {
    console.error('could not fetch accounts');
    if (err) console.error(err.stack);
  });

export const setAccount = (address) => ({ type: 'set account', address });

export const setContract = (contract) => ({ type: 'set contract', contract });

export const fetchContract = () => (dispatch) =>
  api.ethcore.registryAddress()
  .then((address) => {
    const contract = api.newContract(registryAbi, address);
    dispatch(setContract(contract));
    dispatch(fetchFee());
    dispatch(fetchOwner());
  })
  .catch((err) => {
    console.error('could not fetch contract');
    if (err) console.error(err.stack);
  });

export const setFee = (fee) => ({ type: 'set fee', fee });

const fetchFee = () => (dispatch, getState) => {
  const { contract } = getState();
  if (!contract) return;
  contract.instance.fee.call()
  .then((fee) => dispatch(setFee(fee)))
  .catch((err) => {
    console.error('could not fetch fee');
    if (err) console.error(err.stack);
  });
};

export const setOwner = (owner) => ({ type: 'set owner', owner });

export const fetchOwner = () => (dispatch, getState) => {
  const { contract } = getState();
  if (!contract) return;
  contract.instance.owner.call()
  .then((owner) => dispatch(setOwner(owner)))
  .catch((err) => {
    console.error('could not fetch owner');
    if (err) console.error(err.stack);
  });
};
