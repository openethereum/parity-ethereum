import React, { Component, PropTypes } from 'react';
import Menu from 'material-ui/Menu';
import MenuItem from 'material-ui/MenuItem';
import AccountIcon from 'material-ui/svg-icons/action/account-circle';

// import styles from './lookup.css';

const renderAccount = (active) => (account) => (
  <MenuItem
    key={ account.address }
    value={ account.address }
    primaryText={ account.name }
    style={ active && active.address === account.address ? { color: 'red' } : {} }
  />
);

export default class Accounts extends Component {

  static propTypes = {
    actions: PropTypes.object,
    accounts: PropTypes.object,
    account: PropTypes.object
  }

  componentDidMount () {
    // TODO remove this
    this.props.actions.fetchAccounts();
  }

  state = { value: null };

  render () {
    const { open } = this.state;
    const { accounts, account } = this.props;

    return (
      <Menu value={ account ? account.address : null } onChange={ this.onAccountSelect }>
        { Object.values(accounts).map(renderAccount(account)) }
      </Menu>
    );
  }

  onAccountSelect = (e, address) => {
    this.props.actions.setAccount(address);
  };
}
