import registryAbi from './abi/registry.json';
import { newContract, ethcore } from './parity.js';
import * as addresses from './addresses/actions.js';
import * as accounts from './accounts/actions.js';
import * as lookup from './Lookup/actions.js';
import * as events from './events/actions.js';
import * as register from './register/actions.js';
import * as records from './records/actions.js';

export { addresses, accounts, lookup, events, register, records };

export const setContract = (contract) => ({ type: 'set contract', contract });

export const fetchContract = () => (dispatch) =>
  ethcore.registryAddress()
  .then((address) => {
    const contract = newContract(registryAbi, address);
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
