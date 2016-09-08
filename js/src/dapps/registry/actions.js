import registryAbi from './abi/registry.json';
const { api } = window.parity;
import * as lookup from './Lookup/actions.js';

export { lookup };

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
