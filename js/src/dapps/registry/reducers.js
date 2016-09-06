import { handleActions } from 'redux-actions';

import {
  setContract,
  fetchFee, setFee,
  fetchOwner, setOwner
} from './actions';
import registryAbi from '../abi/registry.json';
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

export default handleActions({

  'fetch contract': onFetchContract,
  'set contract': (state, action) =>
    ({ ...state, contract: action.payload }),

  'fetch fee': onFetchFee,
  'set fee': (state, action) =>
    ({ ...state, fee: action.payload }),

  'fetch owner': onFetchOwner,
  'set owner': (state, action) =>
    ({ ...state, owner: action.payload })

}, initialState);
