const initialState = {
  hasAccount: false,
  pending: false,
  posted: []
};

export default (state = initialState, action) => {
  if (action.type === 'accounts select') {
    return { ...state, hasAccount: !!action.address };
  }

  if (action.type === 'register start') {
    return { ...state, pending: true };
  }
  if (action.type === 'register success') {
    return {
      ...state, pending: false,
      posted: state.posted.concat(action.name)
    };
  }
  if (action.type === 'register fail') {
    return { ...state, pending: false };
  }

  return state;
};
