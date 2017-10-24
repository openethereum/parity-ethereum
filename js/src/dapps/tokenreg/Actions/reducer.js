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
  SET_REGISTER_SENDING,
  SET_REGISTER_ERROR,
  REGISTER_RESET,
  REGISTER_COMPLETED,

  SET_QUERY_LOADING,
  SET_QUERY_RESULT,
  SET_QUERY_NOT_FOUND,
  SET_QUERY_META_LOADING,
  SET_QUERY_META,
  QUERY_RESET
} from './actions';

const initialState = {
  register: {
    sending: false,
    error: null,
    complete: false
  },
  query: {
    loading: false,
    data: null,
    notFound: false,
    metaLoading: false,
    metaData: null
  }
};

export default (state = initialState, action) => {
  switch (action.type) {
    case SET_REGISTER_SENDING: {
      const registerState = state.register;

      return {
        ...state,
        register: {
          ...registerState,
          sending: action.isSending
        }
      };
    }

    case REGISTER_COMPLETED: {
      const registerState = state.register;

      return {
        ...state,
        register: {
          ...registerState,
          sending: false,
          complete: true
        }
      };
    }

    case SET_REGISTER_ERROR: {
      const registerState = state.register;

      return {
        ...state,
        register: {
          ...registerState,
          sending: false,
          error: action.error
        }
      };
    }

    case REGISTER_RESET: {
      return {
        ...state,
        register: initialState.register
      };
    }

    case SET_QUERY_LOADING: {
      return {
        ...state,
        query: {
          ...state.query,
          loading: action.isLoading
        }
      };
    }

    case SET_QUERY_RESULT: {
      return {
        ...state,
        query: {
          ...state.query,
          data: action.data
        }
      };
    }

    case SET_QUERY_NOT_FOUND: {
      return {
        ...state,
        query: {
          ...state.query,
          notFound: true
        }
      };
    }

    case SET_QUERY_META_LOADING: {
      return {
        ...state,
        query: {
          ...state.query,
          metaLoading: action.isLoading
        }
      };
    }

    case SET_QUERY_META: {
      return {
        ...state,
        query: {
          ...state.query,
          metaData: action.data
        }
      };
    }

    case QUERY_RESET: {
      return {
        ...state,
        query: {
          ...initialState.query
        }
      };
    }

    default:
      return state;
  }
};
