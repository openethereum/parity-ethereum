// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import { Checkbox } from 'material-ui';

import { IdentityIcon } from '~/ui';

import styles from './newGeth.css';

export default class NewGeth extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    onChange: PropTypes.func.isRequired
  }

  state = {
    available: []
  }

  componentDidMount () {
    this.loadAvailable();
  }

  render () {
    const { available } = this.state;

    if (!available.length) {
      return (
        <div className={ styles.list }>There are currently no importable keys available from the Geth keystore, which are not already available on your Parity instance</div>
      );
    }

    const checkboxes = available.map((account) => {
      const label = (
        <div className={ styles.selection }>
          <div className={ styles.icon }>
            <IdentityIcon
              center inline
              address={ account.address }
            />
          </div>
          <div className={ styles.detail }>
            <div className={ styles.address }>{ account.address }</div>
            <div className={ styles.balance }>{ account.balance } ETH</div>
          </div>
        </div>
      );

      return (
        <Checkbox
          key={ account.address }
          checked={ account.checked }
          label={ label }
          data-address={ account.address }
          onCheck={ this.onSelect }
        />
      );
    });

    return (
      <div className={ styles.list }>
        { checkboxes }
      </div>
    );
  }

  onSelect = (event, checked) => {
    const address = event.target.getAttribute('data-address');

    if (!address) {
      return;
    }

    const { available } = this.state;
    const account = available.find((_account) => _account.address === address);
    account.checked = checked;
    const selected = available.filter((_account) => _account.checked);

    this.setState({
      available
    });

    this.props.onChange(selected.length, selected.map((account) => account.address));
  }

  loadAvailable = () => {
    const { api } = this.context;
    const { accounts } = this.props;

    api.parity
      .listGethAccounts()
      .then((_addresses) => {
        const addresses = (addresses || []).filter((address) => !accounts[address]);

        return Promise
          .all(addresses.map((address) => api.eth.getBalance(address)))
          .then((balances) => {
            this.setState({
              available: addresses.map((address, idx) => {
                return {
                  address,
                  balance: api.util.fromWei(balances[idx]).toFormat(5),
                  checked: false
                };
              })
            });
          });
      })
      .catch((error) => {
        console.error('loadAvailable', error);
      });
  }
}
