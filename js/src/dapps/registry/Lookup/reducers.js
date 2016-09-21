const initialState = {
  pending: false,
  name: '', type: '',
  result: null
};

export default (state = initialState, action) => {
  if (action.type === 'lookup start') {
    return {
      pending: true,
      name: action.name, type: action.entry,
      result: null
    };
  }

  if (action.type === 'lookup error') {
    return {
      pending: false,
      name: initialState.name, type: initialState.type,
      result: null
    };
  }

  if (action.type === 'lookup success') {
    return {
      pending: false,
      name: initialState.name, type: initialState.type,
      result: action.result
    };
  }

  return state;
};
