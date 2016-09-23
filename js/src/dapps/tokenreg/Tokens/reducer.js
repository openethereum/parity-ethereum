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
      let index = action.index;
      let tokens = [].concat(state.tokens);

      tokens[index] = {
        ...tokens[index],
        ...action.tokenData
      };

      return { ...state, tokens: tokens };
    }

    case SET_TOKEN_META: {
      let index = action.index;
      let tokens = [].concat(state.tokens);

      tokens[index] = {
        ...tokens[index],
        meta: action.meta
      };

      return { ...state, tokens: tokens };
    }

    case SET_TOKEN_LOADING: {
      let index = action.index;
      let tokens = [].concat(state.tokens);

      tokens[index] = {
        ...tokens[index],
        isLoading: action.isLoading
      };

      return { ...state, tokens: tokens };
    }

    case SET_TOKEN_META_LOADING: {
      let index = action.index;
      let tokens = [].concat(state.tokens);

      tokens[index] = {
        ...tokens[index],
        isMetaLoading: action.isMetaLoading
      };

      return { ...state, tokens: tokens };
    }

    case SET_TOKEN_PENDING: {
      let index = action.index;
      let tokens = [].concat(state.tokens);

      tokens[index] = {
        ...tokens[index],
        isPending: action.isPending
      };

      return { ...state, tokens: tokens };
    }

    case DELETE_TOKEN: {
      let index = action.index;
      let tokens = [].concat(state.tokens);

      delete tokens[index];

      return { ...state, tokens: tokens };
    }

    default:
      return state;
  }
};
