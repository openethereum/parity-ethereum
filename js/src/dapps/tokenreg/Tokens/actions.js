import { getTokenTotalSupply } from '../utils';

const { sha3, bytesToHex } = window.parity.api.util;

export const SET_TOKENS_LOADING = 'SET_TOKENS_LOADING';
export const setTokensLoading = (isLoading) => ({
  type: SET_TOKENS_LOADING,
  isLoading
});

export const SET_TOKEN_COUNT = 'SET_TOKEN_COUNT';
export const setTokenCount = (tokenCount) => ({
  type: SET_TOKEN_COUNT,
  tokenCount
});

export const SET_TOKEN_DATA = 'SET_TOKEN_DATA';
export const setTokenData = (index, tokenData) => ({
  type: SET_TOKEN_DATA,
  index, tokenData
});

export const SET_TOKEN_META = 'SET_TOKEN_META';
export const setTokenMeta = (index, meta) => ({
  type: SET_TOKEN_META,
  index, meta
});

export const SET_TOKEN_LOADING = 'SET_TOKEN_LOADING';
export const setTokenLoading = (index, isLoading) => ({
  type: SET_TOKEN_LOADING,
  index, isLoading
});

export const SET_TOKEN_META_LOADING = 'SET_TOKEN_META_LOADING';
export const setTokenMetaLoading = (index, isMetaLoading) => ({
  type: SET_TOKEN_META_LOADING,
  index, isMetaLoading
});

export const SET_TOKEN_PENDING = 'SET_TOKEN_PENDING';
export const setTokenPending = (index, isPending) => ({
  type: SET_TOKEN_PENDING,
  index, isPending
});

export const DELETE_TOKEN = 'DELETE_TOKEN';
export const deleteToken = (index) => ({
  type: DELETE_TOKEN,
  index
});

export const loadTokens = () => (dispatch, getState) => {
  console.log('loading tokens...');

  let state = getState();
  let contractInstance = state.status.contract.instance;

  dispatch(setTokensLoading(true));

  contractInstance
    .tokenCount
    .call()
    .then((count) => {
      let tokenCount = parseInt(count);
      console.log(`token count: ${tokenCount}`);
      dispatch(setTokenCount(tokenCount));

      for (let i = 0; i < tokenCount; i++) {
        dispatch(loadToken(i));
      }

      dispatch(setTokensLoading(false));
    })
    .catch((e) => {
      console.error('loadTokens error', e);
    });
};

export const loadToken = (index) => (dispatch, getState) => {
  console.log('loading token', index);

  let state = getState();
  let contractInstance = state.status.contract.instance;

  let userAccounts = state.accounts.list;
  let accountsInfo = state.accounts.accountsInfo;

  dispatch(setTokenLoading(index, true));

  contractInstance
    .token
    .call({}, [ parseInt(index) ])
    .then((result) => {
      let tokenOwner = result[4];

      let isTokenOwner = userAccounts
        .filter(a => a.address === tokenOwner)
        .length > 0;

      let data = {
        index: parseInt(index),
        address: result[0],
        tla: result[1],
        base: result[2].toNumber(),
        name: result[3],
        owner: tokenOwner,
        ownerAccountInfo: accountsInfo[tokenOwner],
        isPending: false,
        isTokenOwner
      };

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
      // If no total supply, must not be a proper token
      if (data.totalSupply === null) {
        dispatch(setTokenData(index, null));
        dispatch(setTokenLoading(index, false));
        return;
      }

      data.totalSupply = data.totalSupply.toNumber();
      console.log(`token loaded: #${index}`, data);
      dispatch(setTokenData(index, data));
      dispatch(setTokenLoading(index, false));

    })
    .catch((e) => {
      dispatch(setTokenData(index, null));
      dispatch(setTokenLoading(index, false));

      if (!e instanceof TypeError) {
        console.error(`loadToken #${index} error`, e);
      }
    });
};

export const queryTokenMeta = (index, query) => (dispatch, getState) => {
  console.log('loading token meta', index, query);

  let state = getState();
  let contractInstance = state.status.contract.instance;

  let key = sha3(query);

  let startDate = Date.now();
  dispatch(setTokenMetaLoading(index, true));

  contractInstance
    .meta
    .call({}, [ index, key ])
    .then((value) => {
      let meta = {
        key, query,
        value: value.find(v => v !== 0) ? bytesToHex(value) : null
      };

      console.log(`token meta loaded: #${index}`, value);
      dispatch(setTokenMeta(index, meta));

      setTimeout(() => {
        dispatch(setTokenMetaLoading(index, false));
      }, 500 - (Date.now() - startDate));
    })
    .catch((e) => {
      console.error(`loadToken #${index} error`, e);
    });
};

export const addTokenMeta = (index, key, value) => (dispatch, getState) => {
  console.log('add token meta', index, key, value);

  let state = getState();
  let contractInstance = state.status.contract.instance;

  let token = state.tokens.tokens.find(t => t.index === index);
  let keyHash = sha3(key);

  let options = {
    from: token.owner
  };

  let values = [ index, keyHash, value ];

  contractInstance
    .setMeta
    .estimateGas(options, values)
    .then((gasEstimate) => {
      options.gas = gasEstimate.mul(1.2).toFixed(0);
      console.log(`transfer: gas estimated as ${gasEstimate.toFixed(0)} setting to ${options.gas}`);

      return contractInstance.setMeta.postTransaction(options, values);
    })
    .catch((e) => {
      console.error(`addTokenMeta #${index} error`, e);
    });
};

export const unregisterToken = (index) => (dispatch, getState) => {
  console.log('unregistering token', index);

  let state = getState();
  let contractInstance = state.status.contract.instance;

  let values = [ index ];
  let options = {
    from: state.accounts.selected.address
  };

  contractInstance
    .unregister
    .estimateGas(options, values)
    .then((gasEstimate) => {
      options.gas = gasEstimate.mul(1.2).toFixed(0);
      console.log(`transfer: gas estimated as ${gasEstimate.toFixed(0)} setting to ${options.gas}`);

      return contractInstance.unregister.postTransaction(options, values);
    })
    .catch((e) => {
      console.error(`unregisterToken #${index} error`, e);
    });
};
