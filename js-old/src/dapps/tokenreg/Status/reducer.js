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

import {
  SET_LOADING,
  SET_CONTRACT_DETAILS,
  SET_GITHUBHINT_CONTRACT,
  SET_SUBSCRIPTION_ID
} from './actions';

const initialState = {
  isLoading: true,
  subscriptionId: null,
  contract: {
    address: null,
    instance: null,
    owner: null,
    isOwner: false,
    fee: null
  },
  githubhint: {
    address: null,
    instance: null
  }
};

export default (state = initialState, action) => {
  switch (action.type) {
    case SET_LOADING:
      return { ...state, isLoading: action.isLoading };

    case SET_SUBSCRIPTION_ID:
      return { ...state, subscriptionId: action.subscriptionId };

    case SET_CONTRACT_DETAILS:
      return { ...state, contract: {
        ...state.contract,
        ...action.details
      } };

    case SET_GITHUBHINT_CONTRACT:
      return { ...state, githubhint: {
        ...state.githubhint,
        ...action.details
      } };

    default:
      return state;
  }
};
