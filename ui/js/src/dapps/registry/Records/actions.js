import { sha3 } from '../parity.js';

export const start = (name, key, value) => ({ type: 'records update start', name, key, value });

export const success = () => ({ type: 'records update success' });

export const fail = () => ({ type: 'records update error' });

export const update = (name, key, value) => (dispatch, getState) => {
  const state = getState();
  const account = state.accounts.selected;
  const contract = state.contract;

  if (!contract || !account) {
    return;
  }

  const fnName = key === 'A' ? 'setAddress' : 'set';
  const fn = contract.functions.find((f) => f.name === fnName);

  name = name.toLowerCase();
  const options = { from: account.address };
  const values = [ sha3(name), key, value ];

  dispatch(start(name, key, value));
  fn.estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2).toFixed(0);
      return fn.postTransaction(options, values);
    })
    .then((data) => {
      dispatch(success());
    }).catch((err) => {
      console.error(`could not update ${key} record of ${name}`);
      if (err) console.error(err.stack);
      dispatch(fail());
    });
};
