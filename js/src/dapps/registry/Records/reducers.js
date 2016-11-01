const initialState = {
  pending: false,
  name: '', type: '', value: ''
};

export default (state = initialState, action) => {
  if (action.type === 'records update start') {
    return {
      ...state,
      pending: true,
      name: action.name, type: action.entry, value: action.value
    };
  }

  if (action.type === 'records update error' || action.type === 'records update success') {
    return {
      ...state,
      pending: false,
      name: initialState.name, type: initialState.type, value: initialState.value
    };
  }

  return state;
};
