// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import styles from './Account.css';

import { IdentityIcon } from '../../../../ui';
import AccountLink from './AccountLink';

class Account extends Component {
  static propTypes = {
    className: PropTypes.string,
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    tokens: PropTypes.object,
    address: PropTypes.string.isRequired,
    chain: PropTypes.string.isRequired,
    balance: PropTypes.object // eth BigNumber, not required since it mght take time to fetch
  };

  state = {
    balanceDisplay: '?'
  };

  componentWillMount () {
    this.updateBalanceDisplay(this.props.balance);
  }

  componentWillReceiveProps (nextProps) {
    if (nextProps.balance === this.props.balance) {
      return;
    }
    this.updateBalanceDisplay(nextProps.balance);
  }

  updateBalanceDisplay (balance) {
    this.setState({
      balanceDisplay: balance ? balance.div(1e18).toFormat(3) : '?'
    });
  }

  render () {
    const { address, chain, className } = this.props;

    return (
      <div className={ `${styles.acc} ${className}` } title={ this.renderTitle() }>
        <AccountLink address={ address } chain={ chain }>
          <IdentityIcon
            center
            address={ address } />
        </AccountLink>
        { this.renderName() }
        { this.renderBalance() }
      </div>
    );
  }

  renderTitle () {
    const { address } = this.props;
    const name = this._retrieveName();

    if (name) {
      return address + ' ' + name;
    }

    return address;
  }

  renderBalance () {
    const { balanceDisplay } = this.state;
    return (
      <span> <strong>{ balanceDisplay }</strong> <small>ETH</small></span>
    );
  }

  renderName () {
    const { address } = this.props;
    const name = this._retrieveName();

    if (!name) {
      return (
        <AccountLink address={ address } chain={ this.props.chain }>
          [{ this.shortAddress(address) }]
        </AccountLink>
      );
    }

    return (
      <AccountLink address={ address } chain={ this.props.chain } >
        <span>
          <span className={ styles.name }>{ name }</span>
          <span className={ styles.address }>[{ this.tinyAddress(address) }]</span>
        </span>
      </AccountLink>
    );
  }

  _retrieveName () {
    const { address, accounts, contacts, tokens } = this.props;
    const account = (accounts || {})[address] || (contacts || {})[address] || (tokens || {})[address];

    return account
      ? account.name
      : null;
  }

  tinyAddress () {
    const { address } = this.props;
    const len = address.length;
    return address.slice(2, 4) + '..' + address.slice(len - 2);
  }

  shortAddress () {
    const { address } = this.props;
    const len = address.length;
    return address.slice(2, 8) + '..' + address.slice(len - 7);
  }
}

function mapStateToProps (state) {
  const { accounts, contacts } = state.personal;
  const { tokens } = state.balances;

  return {
    accounts,
    contacts,
    tokens
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Account);
