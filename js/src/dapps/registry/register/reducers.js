const initialState = {
  contract: null,
  names: []
};

export default (state = initialState, action) => {
  if (action.type === 'register success') {
    return { ...state, names: state.names
      .filter((n) => n !== action.name)
      .concat(action.name)
    };
  }

  return state;
};
