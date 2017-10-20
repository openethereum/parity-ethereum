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
  SET_TOKENS_LOADING,
  SET_TOKEN_COUNT,
  SET_TOKEN_DATA,
  SET_TOKEN_META,
  SET_TOKEN_LOADING,
  SET_TOKEN_META_LOADING,
  SET_TOKEN_PENDING,
  DELETE_TOKEN
} from './actions';

const initialState = {
  isLoading: true,
  tokens: [],
  tokenCount: 0
};

export default (state = initialState, action) => {
  switch (action.type) {
    case SET_TOKENS_LOADING:
      return { ...state, isLoading: action.isLoading };

    case SET_TOKEN_COUNT:
      return { ...state, tokenCount: action.tokenCount };

    case SET_TOKEN_DATA: {
      const index = action.index;
      const tokens = [].concat(state.tokens);

      tokens[index] = {
        ...tokens[index],
        ...action.tokenData
      };

      return { ...state, tokens: tokens };
    }

    case SET_TOKEN_META: {
      const index = action.index;
      const tokens = [].concat(state.tokens);

      tokens[index] = {
        ...tokens[index],
        meta: action.meta
      };

      return { ...state, tokens: tokens };
    }

    case SET_TOKEN_LOADING: {
      const index = action.index;
      const tokens = [].concat(state.tokens);

      tokens[index] = {
        ...tokens[index],
        isLoading: action.isLoading
      };

      return { ...state, tokens: tokens };
    }

    case SET_TOKEN_META_LOADING: {
      const index = action.index;
      const tokens = [].concat(state.tokens);

      tokens[index] = {
        ...tokens[index],
        isMetaLoading: action.isMetaLoading
      };

      return { ...state, tokens: tokens };
    }

    case SET_TOKEN_PENDING: {
      const index = action.index;
      const tokens = [].concat(state.tokens);

      tokens[index] = {
        ...tokens[index],
        isPending: action.isPending
      };

      return { ...state, tokens: tokens };
    }

    case DELETE_TOKEN: {
      const index = action.index;
      const tokens = [].concat(state.tokens);

      delete tokens[index];

      return { ...state, tokens: tokens };
    }

    default:
      return state;
  }
};
