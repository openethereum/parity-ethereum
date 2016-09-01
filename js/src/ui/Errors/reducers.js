function newError (state, action) {
  const { error } = action;

  console.log('newError', state, action);

  return Object.assign({}, state, {
    visible: true,
    message: error.message
  });
}

function closeErrors (state, action) {
  return Object.assign({}, state, {
    visible: false,
    message: null
  });
}

export default function errorReducer (state = {}, action) {
  switch (action.type) {
    case 'newError':
      return newError(state, action);

    case 'closeErrors':
      return closeErrors(state, action);

    default:
      return state;
  }
}
