export const start = (name, from, to) => ({ type: 'events subscribe start', name, from, to });

export const event = (name, event) => ({ type: 'events event', event: { ...event, type: name } });

export const fail = (name) => ({ type: 'events subscribe fail', name });

export const subscribe = (name, from = 0, to = 'latest') =>
  (dispatch, getState) => {
    const { contract } = getState();
    if (!contract || !contract.instance) return;
    if (!contract.instance[name]) return;
    const channel = contract.instance[name];

    dispatch(start(name, from, to));
    channel.subscribe({ fromBlock: from, toBlock: to }, (events) => {
      // TODO there's no error param! the `fail` action is never used
      for (let e of events) {
        const data = {
          key: '' + e.transactionHash + e.logIndex,
          state: e.type,
          block: e.blockNumber,
          index: e.logIndex,
          transaction: e.transactionHash,
          parameters: e.params
        };
        dispatch(event(name, data))
      }
    })
  };
