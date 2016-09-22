import {
  SET_LOADING,
  SET_CONTRACT_DETAILS,
  SET_GITHUBHINT_CONTRACT,
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
    fee: null
  },
  githubhint: {
    address: null,
    instance: null,
    raw: null
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

    case SET_GITHUBHINT_CONTRACT:
      return { ...state, githubhint: {
        ...state.githubhint,
        ...action.details
      } };

    default:
      return state;
  }
};
