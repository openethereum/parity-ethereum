function statusBlockNumber (state, action) {
  const { blockNumber } = action;

  return Object.assign({}, state, { blockNumber });
}

function statusCollection (state, action) {
  const { collection } = action;

  return Object.assign({}, state, collection);
}

function statusLogs (state, action) {
  const { logInfo } = action;

  return Object.assign({}, state, logInfo);
}

export default function statusReducer (state = {}, action) {
  switch (action.type) {
    case 'statusBlockNumber':
      return statusBlockNumber(state, action);

    case 'statusCollection':
      return statusCollection(state, action);

    case 'statusLogs':
      return statusLogs(state, action);

    default:
      return state;
  }
}
