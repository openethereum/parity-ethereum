import {
  SET_LOADING,
  SET_CONTRACT_DETAILS,
  SET_SUBSCRIPTION_ID
} from './actions';

const initialState = {
  isLoading: true,
  subscriptionId: null,
  contract: {
    addres: null,
    instance: null,
    raw: null,
    owner: null,
    isOwner: false,
    fee: null,
  }
};

export default (state = initialState, action) => {
  switch (action.type) {
    case SET_LOADING:
      return { ...state, isLoading: action.isLoading };

    case SET_SUBSCRIPTION_ID:
      return { ...state, subscriptionId: action.subscriptionId };

    case SET_CONTRACT_DETAILS:
      return { ...state, contract: {
        ...state.contract,
        ...action.details
      } };

    default:
      return state;
  }
};
