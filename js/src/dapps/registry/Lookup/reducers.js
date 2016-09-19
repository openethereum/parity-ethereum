const initialState = {
  pending: false,
  name: '', entry: '',
  result: null
};

export default (state = initialState, action) => {
  if (action.type === 'lookup start') {
    return {
      pending: true,
      name: action.name, entry: action.entry,
      result: null
    };
  }

  if (action.type === 'lookup error') {
    return {
      pending: false,
      name: initialState.name, entry: initialState.entry,
      result: null
    };
  }

  if (action.type === 'lookup success') {
    return {
      pending: false,
      name: initialState.name, entry: initialState.entry,
      result: action.result
    };
  }

  return state;
};
