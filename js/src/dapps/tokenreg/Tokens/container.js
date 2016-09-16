import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';

import Tokens from './tokens';

import { loadTokens, queryTokenMeta, unregisterToken } from './actions';

class TokensContainer extends Component {
  static propTypes = {
    isOwner: PropTypes.bool,
    isLoading: PropTypes.bool,
    tokens: PropTypes.array,
    tokenCount: PropTypes.number
  };

  componentDidMount() {
    this.props.onLoadTokens();
  }

  render() {
    return (<Tokens
      { ...this.props }
    />);
  }
}

const mapStateToProps = (state) => {
  const { isLoading, tokens, tokenCount } = state.tokens;
  const { isOwner } = state.status.contract;

  return { isLoading, tokens, tokenCount, isOwner };
};

const mapDispatchToProps = (dispatch) => {
  return {
    onLoadTokens: () => {
      dispatch(loadTokens());
    },

    handleMetaLookup: (index, query) => {
      dispatch(queryTokenMeta(index, query))
    },

    handleUnregister: (index) => {
      dispatch(unregisterToken(index));
    }
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TokensContainer);
