import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';

import Tokens from './tokens';

import { loadTokens, queryTokenMeta } from './actions';

class TokensContainer extends Component {
  static propTypes = {
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

  return { isLoading, tokens, tokenCount };
};

const mapDispatchToProps = (dispatch) => {
  return {
    onLoadTokens: () => {
      dispatch(loadTokens());
    },

    handleMetaLookup: (index, query) => {
      dispatch(queryTokenMeta(index, query))
    }
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TokensContainer);
