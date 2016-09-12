import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import styles from './balances.css';

class Balances extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    account: PropTypes.object,
    balances: PropTypes.object
  }

  render () {
    const { api } = this.context;
    const { account, balances } = this.props;

    if (!balances[account.address]) {
      return null;
    }

    let body = balances[account.address].tokens
      .filter((balance) => new BigNumber(balance.value).gt(0))
      .map((balance) => {
        const token = balance.token;
        const value = token.format
          ? new BigNumber(balance.value).div(new BigNumber(token.format)).toFormat(3)
          : api.format.fromWei(balance.value).toFormat(3);

        return (
          <div
            className={ styles.balance }
            key={ token.tag }>
            <img
              src={ token.images.small }
              alt={ token.name } />
            <div>{ value }<small> { token.tag }</small></div>
          </div>
        );
      });

    if (!body.length) {
      body = (
        <div className={ styles.empty }>
          There are no balances associated with this account
        </div>
      );
    }

    return (
      <div className={ styles.balances }>
        { body }
      </div>
    );
  }
}

function mapStateToProps (state) {
  const { balances } = state.balances;

  return {
    balances
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Balances);
