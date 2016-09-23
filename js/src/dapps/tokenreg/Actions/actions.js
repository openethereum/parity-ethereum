import { getTokenTotalSupply } from '../utils';

export const SET_REGISTER_SENDING = 'SET_REGISTER_SENDING';
export const setRegisterSending = (isSending) => ({
  type: SET_REGISTER_SENDING,
  isSending
});

export const SET_REGISTER_ERROR = 'SET_REGISTER_ERROR';
export const setRegisterError = (e) => ({
  type: SET_REGISTER_ERROR,
  error: e
});

export const REGISTER_RESET = 'REGISTER_RESET';
export const registerReset = () => ({
  type: REGISTER_RESET
});

export const REGISTER_COMPLETED = 'REGISTER_COMPLETED';
export const registerCompleted = () => ({
  type: REGISTER_COMPLETED
});

export const registerToken = (tokenData) => (dispatch, getState) => {
  console.log('registering token', tokenData);

  let state = getState();
  let contractInstance = state.status.contract.instance;
  let fee = state.status.contract.fee;

  const { address, base, name, tla } = tokenData;

  dispatch(setRegisterSending(true));

  let values = [ address, tla, base, name ];
  let options = {
    from: state.accounts.selected.address,
    value: fee
  };

  Promise.resolve()
    .then(() => {
      return contractInstance
        .fromTLA.call({}, [ tla ])
        .then((result) => {
          throw new Error(`A Token has already been registered with the TLA ${tla}`);
        }, () => {});
    })
    .then(() => {
      return contractInstance
        .fromAddress.call({}, [ address ])
        .then((result) => {
          throw new Error(`A Token has already been registered with the Address ${address}`);
        }, () => {});
    })
    .then(() => {
      return contractInstance
        .register
        .estimateGas(options, values);
    })
    .then((gasEstimate) => {
      options.gas = gasEstimate.mul(1.2).toFixed(0);
      console.log(`transfer: gas estimated as ${gasEstimate.toFixed(0)} setting to ${options.gas}`);

      return contractInstance.register.postTransaction(options, values);
    })
    .then((result) => {
      dispatch(registerCompleted());
    })
    .catch((e) => {
      console.error('registerToken error', e);
      dispatch(setRegisterError(e));
    });
};

export const SET_QUERY_LOADING = 'SET_QUERY_LOADING';
export const setQueryLoading = (isLoading) => ({
  type: SET_QUERY_LOADING,
  isLoading
});

export const SET_QUERY_RESULT = 'SET_QUERY_RESULT';
export const setQueryResult = (data) => ({
  type: SET_QUERY_RESULT,
  data
});

export const SET_QUERY_NOT_FOUND = 'SET_QUERY_NOT_FOUND';
export const setQueryNotFound = () => ({
  type: SET_QUERY_NOT_FOUND
});

export const QUERY_RESET = 'QUERY_RESET';
export const queryReset = () => ({
  type: QUERY_RESET
});

export const queryToken = (key, query) => (dispatch, getState) => {
  let state = getState();
  let contractInstance = state.status.contract.instance;

  let contractFunc = (key === 'tla') ? 'fromTLA' : 'fromAddress';

  dispatch(setQueryLoading(true));

  contractInstance[contractFunc]
    .call({}, [ query ])
    .then((result) => {
      let data = {
        id: result[0].toNumber(),
        base: result[2].toNumber(),
        name: result[3],
        owner: result[4]
      };

      if (key === 'tla') {
        data.tla = query;
        data.address = result[1];
      }

      if (key === 'address') {
        data.address = query;
        data.tla = result[1];
      }

      return data;
    })
    .then(data => {
      return getTokenTotalSupply(data.address)
        .then(totalSupply => {
          data.totalSupply = totalSupply;
          return data;
        });
    })
    .then(data => {
      if (data.totalSupply === null) {
        dispatch(setQueryNotFound());
        dispatch(setQueryLoading(false));

        return false;
      }

      data.totalSupply = data.totalSupply.toNumber();
      dispatch(setQueryResult(data));
      dispatch(setQueryLoading(false));
    }, () => {
      dispatch(setQueryNotFound());
      dispatch(setQueryLoading(false));
    });
};
