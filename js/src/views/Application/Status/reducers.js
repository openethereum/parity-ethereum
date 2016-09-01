function updateStatus (state, action) {
  const { blockNumber, clientVersion, netPeers, netChain } = action.status;

  return Object.assign({}, state, {
    blockNumber,
    clientVersion,
    netPeers,
    netChain
  });
}

export default function statusReducer (state = {}, action) {
  switch (action.type) {
    case 'updateStatus':
      return updateStatus(state, action);

    default:
      return state;
  }
}
