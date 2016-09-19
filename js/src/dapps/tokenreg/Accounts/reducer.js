import {
  SET_ACCOUNTS,
  SET_SELECTED_ACCOUNT
} from './actions';

const initialState = {
  list: [],
  selected: null
};

export default (state = initialState, action) => {
  switch (action.type) {
    case SET_ACCOUNTS:
      return {
        ...state,
        list: [].concat(action.accounts)
      };

    case SET_SELECTED_ACCOUNT: {
      let address = action.address;
      let account = state.list.find(a => a.address === address);

      return {
        ...state,
        selected: account
      };
    }

    default:
      return state;
  }
};

