import { SET_LOADING, SET_CONTRACT_DETAILS } from './actions';

const initialState = {
  isLoading: true,
  contract: {
    addres: null,
    instance: null,
    owner: null,
    fee: null,
  }
};

export default (state = initialState, action) => {
  switch (action.type) {
    case SET_LOADING:
      return { ...state, isLoading: action.isLoading };

    case SET_CONTRACT_DETAILS:
      return { ...state, contract: {
        ...state.contract,
        ...action.details
      } };

    default:
      return state;
  }
};
