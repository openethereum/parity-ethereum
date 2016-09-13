const sha3 = window.parity.api.format.sha3;
const toWei = window.parity.api.format.toWei;

export const start = (name) => ({ type: 'register start', name });

export const success = (name) => ({ type: 'register success', name });

export const fail = (name) => ({ type: 'register error', name });

export const register = (name) => (dispatch, getState) => {
  const { contract, account } = getState();
  if (!contract || !account) return;
  const reserve = contract.functions
    .find((f) => f.name === 'reserve');

  const options = {
    from: account.address,
    value: toWei(1).toString()
  };
  const values = [ sha3(name) ];

  reserve.estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2).toFixed(0);
      dispatch(start(name));
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
