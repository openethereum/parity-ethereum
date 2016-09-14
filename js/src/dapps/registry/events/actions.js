export const start = (name, from, to) => ({ type: 'events subscribe start', name, from, to });
export const fail = (name) => ({ type: 'events subscribe fail', name });
export const success = (name, subscription) => ({ type: 'events subscribe success', name, subscription });

export const event = (name, event) => ({ type: 'events event', name, event });

export const subscribe = (name, from = 0, to = 'latest') =>
  (dispatch, getState) => {
    const { contract } = getState();
    if (!contract) return;
    const opt = { fromBlock: from, toBlock: to };

    dispatch(start(name, from, to));
    const subscription = contract.subscribe(name, opt, (err, events) => {
      if (err) {
        console.error(`could not subscribe to event ${name}.`);
        console.error(err);
        return dispatch(fail(name));
      }
      dispatch(success(name, subscription));

      for (let e of events) {
        const data = {
          type: name,
          key: '' + e.transactionHash + e.logIndex,
          state: e.type,
          block: e.blockNumber,
          index: e.logIndex,
          transaction: e.transactionHash,
          parameters: e.params
        };
        console.warn('event', data);
        dispatch(event(name, data));
      }
    });
  };

export const unsubscribe = (name) =>
  (dispatch, getState) => {
    const state = getState();
    if (!state.contract) return;
    const subscriptions = state.events.subscriptions;
    if (!(name in subscriptions) || subscriptions[name] === null) return;

    state.contract.unsubscribe(subscriptions[name]);
    dispatch({ type: 'events unsubscribe', name });
  };
