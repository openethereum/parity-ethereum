const initialState = {
  pending: false,
  name: null, key: null,
  result: null
};

export default (state = initialState, action) => {
  if (action.type === 'lookup start')
    return {
      pending: true,
      name: action.name, key: action.key,
      result: null
    };

  if (action.type === 'lookup error')
    return {
      pending: false,
      name: null, key: null,
      result: null
    };

  if (action.type === 'lookup success')
    return {
      pending: false,
      name: null, key: null,
      result: action.result
    };

  return state;
};
