export const start = (name, from, to) => ({ type: 'events subscribe start', name, from, to });

export const event = (name, event) => ({ type: 'events event', event: { ...event, type: name } });

export const fail = (name) => ({ type: 'events subscribe fail', name });

export const subscribe = (name, from = 0, to = 'latest') =>
  (dispatch, getState) => {
    const { contract } = getState();
    if (!contract || !contract.instance) return;

    dispatch(start(name, from, to));
    contract.subscribe(name, { fromBlock: from, toBlock: to }, (err, events) => {
      if (err) {
        console.error(`could not subscribe to event ${name}.`);
        console.error(err);
        return dispatch(fail(name));
      }
      for (let e of events) {
        const data = {
          key: '' + e.transactionHash + e.logIndex,
          state: e.type,
          block: e.blockNumber,
          index: e.logIndex,
          transaction: e.transactionHash,
          parameters: e.params
        };
        dispatch(event(name, data));
      }
    });
  };
