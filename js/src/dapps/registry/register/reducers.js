const initialState = {
  pending: false,
  posted: []
};

export default (state = initialState, action) => {
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
