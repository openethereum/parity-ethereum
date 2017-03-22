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

import Contracts from '~/contracts';

import { loadToken, setTokenPending, deleteToken, setTokenData } from '../Tokens/actions';
import { api } from '../parity';

export const SET_LOADING = 'SET_LOADING';
export const setLoading = (isLoading) => ({
  type: SET_LOADING,
  isLoading
});

export const FIND_CONTRACT = 'FIND_CONTRACT';
export const loadContract = () => (dispatch) => {
  dispatch(setLoading(true));

  const { tokenReg, githubHint } = new Contracts(api);

  return Promise
    .all([
      tokenReg.getContract(),
      githubHint.getContract()
    ])
    .then(([ tokenRegContract, githubHintContract ]) => {
      dispatch(setContractDetails({
        address: tokenRegContract.address,
        instance: tokenRegContract.instance,
        raw: tokenRegContract
      }));

      dispatch(setGithubhintDetails({
        address: githubHintContract.address,
        instance: githubHintContract.instance,
        raw: githubHintContract
      }));

      dispatch(loadContractDetails());
      dispatch(subscribeEvents());
    })
    .catch((error) => {
      throw error;
    });
};

export const LOAD_CONTRACT_DETAILS = 'LOAD_CONTRACT_DETAILS';
export const loadContractDetails = () => (dispatch, getState) => {
  const state = getState();

  const { instance } = state.status.contract;

  Promise
    .all([
      api.eth.accounts(),
      instance.owner.call(),
      instance.fee.call()
    ])
    .then(([accounts, owner, fee]) => {
      const isOwner = accounts.filter(a => a === owner).length > 0;

      dispatch(setContractDetails({
        fee,
        owner,
        isOwner
      }));

      dispatch(setLoading(false));
    })
    .catch((error) => {
      console.error('loadContractDetails error', error);
    });
};

export const SET_CONTRACT_DETAILS = 'SET_CONTRACT_DETAILS';
export const setContractDetails = (details) => ({
  type: SET_CONTRACT_DETAILS,
  details
});

export const SET_GITHUBHINT_CONTRACT = 'SET_GITHUBHINT_CONTRACT';
export const setGithubhintDetails = (details) => ({
  type: SET_GITHUBHINT_CONTRACT,
  details
});

export const subscribeEvents = () => (dispatch, getState) => {
  const state = getState();

  const { raw } = state.status.contract;
  const previousSubscriptionId = state.status.subscriptionId;

  if (previousSubscriptionId) {
    raw.unsubscribe(previousSubscriptionId);
  }

  raw
    .subscribe(null, {
      fromBlock: 'latest',
      toBlock: 'pending',
      limit: 50
    }, (error, logs) => {
      if (error) {
        console.error('setupFilters', error);
        return;
      }

      if (!logs || logs.length === 0) {
        return;
      }

      logs.forEach(log => {
        const event = log.event;
        const type = log.type;
        const params = log.params;

        if (event === 'Registered' && type === 'pending') {
          return dispatch(setTokenData(params.id.value.toNumber(), {
            tla: '...',
            base: -1,
            address: params.addr.value,
            name: params.name.value,
            isPending: true
          }));
        }

        if (event === 'Registered' && type === 'mined') {
          return dispatch(loadToken(params.id.value.toNumber()));
        }

        if (event === 'Unregistered' && type === 'pending') {
          return dispatch(setTokenPending(params.id.value.toNumber(), true));
        }

        if (event === 'Unregistered' && type === 'mined') {
          return dispatch(deleteToken(params.id.value.toNumber()));
        }

        if (event === 'MetaChanged' && type === 'pending') {
          return dispatch(setTokenData(
            params.id.value.toNumber(),
            { metaPending: true, metaMined: false }
          ));
        }

        if (event === 'MetaChanged' && type === 'mined') {
          setTimeout(() => {
            dispatch(setTokenData(
              params.id.value.toNumber(),
              { metaPending: false, metaMined: false }
            ));
          }, 5000);

          return dispatch(setTokenData(
            params.id.value.toNumber(),
            { metaPending: false, metaMined: true }
          ));
        }

        console.warn('unknown log event', log);
      });
    })
    .then((subscriptionId) => {
      dispatch(setSubscriptionId(subscriptionId));
    });
};

export const SET_SUBSCRIPTION_ID = 'SET_SUBSCRIPTION_ID';
export const setSubscriptionId = subscriptionId => ({
  type: SET_SUBSCRIPTION_ID,
  subscriptionId
});
