// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import {
  registry as registryAbi,
  tokenreg as tokenregAbi,
  githubhint as githubhintAbi
} from '../../../contracts/abi';

import { loadToken, setTokenPending, deleteToken, setTokenData } from '../Tokens/actions';

const { api } = window.parity;

export const SET_LOADING = 'SET_LOADING';
export const setLoading = (isLoading) => ({
  type: SET_LOADING,
  isLoading
});

export const FIND_CONTRACT = 'FIND_CONTRACT';
export const loadContract = () => (dispatch) => {
  dispatch(setLoading(true));

  api.ethcore
    .registryAddress()
    .then((registryAddress) => {
      console.log(`registry found at ${registryAddress}`);
      const registry = api.newContract(registryAbi, registryAddress).instance;

      return Promise.all([
        registry.getAddress.call({}, [api.util.sha3('tokenreg'), 'A']),
        registry.getAddress.call({}, [api.util.sha3('githubhint'), 'A'])
      ]);
    })
    .then(([ tokenregAddress, githubhintAddress ]) => {
      console.log(`tokenreg was found at ${tokenregAddress}`);

      const tokenregContract = api
        .newContract(tokenregAbi, tokenregAddress);

      const githubhintContract = api
        .newContract(githubhintAbi, githubhintAddress);

      dispatch(setContractDetails({
        address: tokenregAddress,
        instance: tokenregContract.instance,
        raw: tokenregContract
      }));

      dispatch(setGithubhintDetails({
        address: githubhintAddress,
        instance: githubhintContract.instance,
        raw: githubhintContract
      }));

      dispatch(loadContractDetails());
      dispatch(subscribeEvents());
    })
    .catch((error) => {
      console.error('loadContract error', error);
    });
};

export const LOAD_CONTRACT_DETAILS = 'LOAD_CONTRACT_DETAILS';
export const loadContractDetails = () => (dispatch, getState) => {
  const state = getState();

  const instance = state.status.contract.instance;

  Promise
    .all([
      api.personal.listAccounts(),
      instance.owner.call(),
      instance.fee.call()
    ])
    .then(([accounts, owner, fee]) => {
      console.log(`owner as ${owner}, fee set at ${fee.toFormat()}`);

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

  const contract = state.status.contract.raw;
  const previousSubscriptionId = state.status.subscriptionId;

  if (previousSubscriptionId) {
    contract.unsubscribe(previousSubscriptionId);
  }

  contract
    .subscribe(null, {
      fromBlock: 'latest',
      toBlock: 'pending',
      limit: 50
    }, (error, logs) => {
      if (error) {
        console.error('setupFilters', error);
        return;
      }

      if (!logs || logs.length === 0) return;

      logs.forEach(log => {
        const event = log.event;
        const type = log.type;
        const params = log.params;

        if (event === 'Registered' && type === 'pending') {
          return dispatch(setTokenData(params.id.toNumber(), {
            tla: '...',
            base: -1,
            address: params.addr,
            name: params.name,
            isPending: true
          }));
        }

        if (event === 'Registered' && type === 'mined') {
          return dispatch(loadToken(params.id.toNumber()));
        }

        if (event === 'Unregistered' && type === 'pending') {
          return dispatch(setTokenPending(params.id.toNumber(), true));
        }

        if (event === 'Unregistered' && type === 'mined') {
          return dispatch(deleteToken(params.id.toNumber()));
        }

        if (event === 'MetaChanged' && type === 'pending') {
          return dispatch(setTokenData(
            params.id.toNumber(),
            { metaPending: true, metaMined: false }
          ));
        }

        if (event === 'MetaChanged' && type === 'mined') {
          setTimeout(() => {
            dispatch(setTokenData(
              params.id.toNumber(),
              { metaPending: false, metaMined: false }
            ));
          }, 5000);

          return dispatch(setTokenData(
            params.id.toNumber(),
            { metaPending: false, metaMined: true }
          ));
        }

        console.log('new log event', log);
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
