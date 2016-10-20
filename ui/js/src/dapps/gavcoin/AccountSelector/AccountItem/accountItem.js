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

import IdentityIcon from '../../IdentityIcon';

import styles from './accountItem.css';

export default class AccountItem extends Component {
  static propTypes = {
    account: PropTypes.object,
    gavBalance: PropTypes.bool
  };

  render () {
    const { account, gavBalance } = this.props;

    let balance;
    let token;

    if (gavBalance) {
      if (account.gavBalance) {
        balance = account.gavBalance;
        token = 'GAV';
      }
    } else {
      if (account.ethBalance) {
        balance = account.ethBalance;
        token = 'ETH';
      }
    }

    return (
      <div className={ styles.account }>
        <div className={ styles.image }>
          <IdentityIcon address={ account.address } />
        </div>
        <div className={ styles.details }>
          <div className={ styles.name }>
            { account.name || account.address }
          </div>
          <div className={ styles.balance }>
            { balance }<small> { token }</small>
          </div>
        </div>
      </div>
    );
  }
}
