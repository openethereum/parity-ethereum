const sha3 = window.parity.api.format.sha3;

export const start = (name, key) => ({ type: 'lookup start', name, key });

export const success = (address) => ({ type: 'lookup success', result: address });

export const fail = () => ({ type: 'lookup error' });

export const lookup = (name, key) => (dispatch, getState) => {
  const { contract } = getState();
  if (!contract) return;
  const getAddress = contract.functions
    .find((f) => f.name === 'getAddress');

  dispatch(start(name, key));
  getAddress.call({}, [sha3(name), key])
    .then((address) => dispatch(success(address)))
    .catch((err) => {
      console.error(`could not lookup ${key} for ${name}`);
      if (err) console.error(err.stack);
      dispatch(fail());
    });
};
