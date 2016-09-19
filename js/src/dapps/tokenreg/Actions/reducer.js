import {
  SET_REGISTER_SENDING,
  SET_REGISTER_ERROR,
  REGISTER_RESET,
  REGISTER_COMPLETED
} from './actions';

const initialState = {
  register: {
    sending: false,
    error: null,
    complete: false
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

    default:
      return state;
  }
};
