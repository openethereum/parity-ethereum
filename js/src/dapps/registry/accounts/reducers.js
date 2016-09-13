const initialState = {
  all: {},
  selected: null
};

export default (state = initialState, action) => {
  if (action.type === 'accounts set')
    return { ...state, all: action.accounts };

  if (action.type === 'accounts select' && state.all && state.all[action.address])
    return { ...state, selected: state.all[action.address] };

  return state;
};
