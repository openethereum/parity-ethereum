// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import { URL_TYPE } from '../Inputs/validation';
import { getTokenTotalSupply, urlToHash } from '../utils';
import { api } from '../parity';

const { bytesToHex } = api.util;

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
  const state = getState();
  const contractInstance = state.status.contract.instance;

  dispatch(setTokensLoading(true));

  contractInstance
    .tokenCount
    .call()
    .then((count) => {
      const tokenCount = parseInt(count);

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
  const state = getState();
  const contractInstance = state.status.contract.instance;
  const userAccounts = state.accounts.list;
  const accountsInfo = state.accounts.accountsInfo;

  dispatch(setTokenLoading(index, true));

  contractInstance
    .token
    .call({}, [ parseInt(index) ])
    .then((result) => {
      const tokenOwner = result[4];
      const isTokenOwner = userAccounts
        .filter(a => a.address === tokenOwner)
        .length > 0;
      const data = {
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

      dispatch(setTokenData(index, data));
      dispatch(setTokenLoading(index, false));
    })
    .catch((e) => {
      dispatch(setTokenData(index, null));
      dispatch(setTokenLoading(index, false));

      if (!(e instanceof TypeError)) {
        console.error(`loadToken #${index} error`, e);
      }
    });
};

export const queryTokenMeta = (index, query) => (dispatch, getState) => {
  const state = getState();
  const contractInstance = state.status.contract.instance;
  const startDate = Date.now();

  dispatch(setTokenMetaLoading(index, true));

  contractInstance
    .meta
    .call({}, [ index, query ])
    .then((value) => {
      const meta = {
        query,
        value: value.find(v => v !== 0) ? bytesToHex(value) : null
      };

      dispatch(setTokenMeta(index, meta));

      setTimeout(() => {
        dispatch(setTokenMetaLoading(index, false));
      }, 500 - (Date.now() - startDate));
    })
    .catch((e) => {
      console.error(`loadToken #${index} error`, e);
    });
};

export const addTokenMeta = (index, key, value, validationType) => (dispatch, getState) => {
  const state = getState();

  const contractInstance = state.status.contract.instance;
  const ghhInstance = state.status.githubhint.instance;

  const token = state.tokens.tokens.find(t => t.index === index);
  const options = { from: token.owner };
  let valuesPromise;

  // Get the right values (could be a hashed URL from GHH)
  if (validationType === URL_TYPE) {
    valuesPromise = addGithubhintURL(ghhInstance, options, value)
      .then((hash) => [ index, key, hash ]);
  } else {
    valuesPromise = Promise.resolve([ index, key, value ]);
  }

  return valuesPromise
    .then((values) => {
      return contractInstance
        .setMeta
        .estimateGas(options, values)
        .then((gasEstimate) => {
          options.gas = gasEstimate.mul(1.2).toFixed(0);
          return contractInstance.setMeta.postTransaction(options, values);
        });
    })
    .catch((e) => {
      console.error(`addTokenMeta: #${index} error`, e);
    });
};

export const addGithubhintURL = (ghhInstance, _options, url) => {
  return urlToHash(ghhInstance, url)
    .then((result) => {
      const { hash, registered } = result;

      if (registered) {
        return hash;
      }

      const options = { from: _options.from };
      const values = [ hash, url ];

      ghhInstance
        .hintURL
        .estimateGas(options, values)
        .then((gasEstimate) => {
          options.gas = gasEstimate.mul(1.2).toFixed(0);
          return ghhInstance.hintURL.postTransaction(options, values);
        })
        .catch((error) => {
          console.error(`registering "${url}" to GHH`, error);
        });

      return hash;
    });
};

export const unregisterToken = (index) => (dispatch, getState) => {
  const { contract } = getState().status;
  const { instance, owner } = contract;
  const values = [ index ];
  const options = {
    from: owner
  };

  instance
    .unregister
    .estimateGas(options, values)
    .then((gasEstimate) => {
      options.gas = gasEstimate.mul(1.2).toFixed(0);
      return instance.unregister.postTransaction(options, values);
    })
    .catch((e) => {
      console.error(`unregisterToken #${index} error`, e);
    });
};
