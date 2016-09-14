import React, { Component, PropTypes } from 'react';

import Token from './Token';
import Loading from '../Loading';

import styles from './tokens.css';

export default class Tokens extends Component {
  static propTypes = {
    handleMetaLookup: PropTypes.func,
    isLoading: PropTypes.bool,
    tokens: PropTypes.array,
    tokenCount: PropTypes.number
  };

  render () {
    const { isLoading, tokens, tokenCount } = this.props;

    let loading = isLoading ? (<Loading size={2} />) : null;

    return (
      <div className={ styles.tokens }>
        { this.renderTokens(tokens) }
        { loading }
      </div>
    );
  }

  renderTokens(tokens) {
    return tokens.map((token, index) => (
      <Token
        { ...token }
        handleMetaLookup={ this.props.handleMetaLookup }
        key={index} />
    ));
  }
}
