import React, { Component, PropTypes } from 'react';

import { Checkbox } from 'material-ui';

import Api from '../../../api';
import IdentityIcon from '../../../ui/IdentityIcon';

import styles from './style.css';

export default class NewGeth extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    accounts: PropTypes.array
  }

  static propTypes = {
    onChange: PropTypes.func.isRequired
  }

  state = {
    available: []
  }

  componentDidMount () {
    this.loadAvailable();
  }

  render () {
    if (!this.state.available.length) {
      return (
        <div className={ styles.list }>There are currently no importable keys available from the Geth keystore, which are not already available on your Parity instance</div>
      );
    }
    const checkboxes = this.state.available.map((account) => {
      const label = (
        <div className={ styles.selection }>
          <div className={ styles.icon }>
            <IdentityIcon
              center inline
              address={ account.address } />
          </div>
          <div className={ styles.detail }>
            <div className={ styles.address }>{ account.address }</div>
            <div className={ styles.balance }>{ account.balance } ΞTH</div>
          </div>
        </div>
      );

      return (
        <Checkbox
          key={ account.address }
          checked={ account.checked }
          label={ label }
          data-address={ account.address }
          onCheck={ this.onSelect } />
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

    const available = this.state.available;
    const account = available.find((_account) => _account.address === address);
    account.checked = checked;
    const selected = available.filter((_account) => _account.checked);

    this.setState({
      available
    });

    this.props.onChange(selected.length, selected.map((account) => account.address));
  }

  loadAvailable = () => {
    const api = this.context.api;

    api.personal
      .listGethAccounts()
      .then((addresses) => {
        return Promise
          .all((addresses || []).map((address) => {
            return api.eth.getBalance(address);
          }))
          .then((balances) => {
            this.setState({
              available: addresses
                .filter((address) => {
                  return !this.context.accounts.find((account) => account.address === address);
                })
                .map((address, idx) => {
                  return {
                    address,
                    balance: Api.format.fromWei(balances[idx]).toFormat(5),
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
