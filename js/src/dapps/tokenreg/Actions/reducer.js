import {
  SET_REGISTER_SENDING,
  SET_REGISTER_ERROR,
  REGISTER_RESET,
  REGISTER_COMPLETED,

  SET_QUERY_LOADING,
  SET_QUERY_RESULT,
  SET_QUERY_NOT_FOUND,
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
    notFound: false
  }
};

export default (state = initialState, action) => {
  switch (action.type) {
    case SET_REGISTER_SENDING: {
      let registerState = state.register;

      return {
        ...state,
        register: {
          ...registerState,
          sending: action.isSending
        }
      };
    }

    case REGISTER_COMPLETED: {
      let registerState = state.register;

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
      let registerState = state.register;

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
