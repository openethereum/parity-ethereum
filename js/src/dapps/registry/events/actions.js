import { getBlockByNumber } from '../parity.js';

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

    contract
      .subscribe(name, opt, (error, events) => {
        if (error) {
          console.error(`error receiving events for ${name}`, error);
          return;
        }

        events.forEach((e) => {
          getBlockByNumber(e.blockNumber)
          .then((block) => {
            const data = {
              type: name,
              key: '' + e.transactionHash + e.logIndex,
              state: e.type,
              block: e.blockNumber,
              index: e.logIndex,
              transaction: e.transactionHash,
              parameters: e.params,
              timestamp: block.timestamp
            };
            dispatch(event(name, data));
          })
          .catch((err) => {
            console.error(`could not fetch block ${e.blockNumber}.`);
            console.error(err);
          });
        });
      })
      .then((subscriptionId) => {
        dispatch(success(name, subscriptionId));
      })
      .catch((error) => {
        console.error('event subscription failed', error);
        dispatch(fail(name));
      });
  };

export const unsubscribe = (name) =>
  (dispatch, getState) => {
    const state = getState();
    if (!state.contract) return;
    const subscriptions = state.events.subscriptions;
    if (!(name in subscriptions) || subscriptions[name] === null) return;

    state.contract
      .unsubscribe(subscriptions[name])
      .then(() => {
        dispatch({ type: 'events unsubscribe', name });
      })
      .catch((error) => {
        console.error('event unsubscribe failed', error);
      });
  };
