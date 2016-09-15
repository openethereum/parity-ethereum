import { sha3, toWei } from '../parity.js';

export const start = (name) => ({ type: 'register start', name });

export const success = (name) => ({ type: 'register success', name });

export const fail = (name) => ({ type: 'register fail', name });

export const register = (name) => (dispatch, getState) => {
  const state = getState();
  const account = state.accounts.selected;
  const contract = state.contract;
  if (!contract || !account) return;
  const reserve = contract.functions.find((f) => f.name === 'reserve');

  name = name.toLowerCase();
  const options = {
    from: account.address,
    value: toWei(1).toString()
  };
  const values = [ sha3(name) ];

  dispatch(start(name));
  reserve.estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2).toFixed(0);
      return reserve.postTransaction(options, values);
    })
    .then((data) => {
      dispatch(success(name));
    }).catch((err) => {
      console.error(`could not reserve ${name}`);
      if (err) console.error(err.stack);
      dispatch(fail(name));
    });
};
