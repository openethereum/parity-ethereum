import {
  setContract,
  fetchFee, setFee,
  fetchOwner, setOwner
} from './actions';
import registryAbi from './abi/registry.json';
const { api } = window.parity;

const initialState = {
  contract: null,
  fee: null,
  owner: null
};

const onFetchContract = (state, action) => (dispatch) =>
  api.ethcore.registryAddress().then(
    (address) => {
      const contract = api.newContract(registryAbi, address);
      dispatch(setContract(contract));
      dispatch(fetchFee());
      dispatch(fetchOwner());
    },
    () => console.error('could not fetch contract')
  );

const onFetchFee = (state, action) => (dispatch) =>
  state.contract.fee.call().then(
    (fee) => dispatch(setFee(fee)),
    () => console.error('could not fetch fee')
  );

const onFetchOwner = (state, action) => (dispatch) =>
  state.contract.owner.call().then(
    (owner) => dispatch(setOwner(owner)),
    () => console.error('could not fetch owner')
  );

export default (state = initialState, action) => {
  if (action.type === 'fetch contract')
    return onFetchContract(state, action);
  if (action.type === 'set contract')
    return { ...state, contract: action.contract };

  if (action.type === 'fetch fee')
    return onFetchFee(state, action);
  if (action.type === 'set fee')
    return { ...state, fee: action.fee };

  if (action.type === 'fetch owner')
    return onFetchOwner(state, action);
  if (action.type === 'set owner')
    return { ...state, owner: action.owner };

  return state;
};
