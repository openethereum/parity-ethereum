import React, { Component, PropTypes } from 'react';
import IconMenu from 'material-ui/IconMenu';
import IconButton from 'material-ui/IconButton/IconButton';
import AccountIcon from 'material-ui/svg-icons/action/account-circle';
import MenuItem from 'material-ui/MenuItem';
import IdentityIcon from '../../../ui/IdentityIcon';

import styles from './accounts.css';

const renderAccount = (active) => (account) => {
  const selected = active && active.address === account.address;
  return (
    <MenuItem
      key={ account.address } value={ account.address }
      checked={ selected } insetChildren={ !selected }
    >
      <IdentityIcon className={ styles.menuIcon } inline center address={ account.address } />
      <span className={ styles.menuText }>{ account.name }</span>
    </MenuItem>
  );
};

export default class Accounts extends Component {

  static propTypes = {
    actions: PropTypes.object.isRequired,
    all: PropTypes.object.isRequired,
    selected: PropTypes.object
  }

  render () {
    const { all, selected } = this.props;

    const accountsButton = (
      <IconButton className={ styles.button }>
        { selected
          ? (<IdentityIcon className={ styles.icon } center address={ selected.address } />)
          : (<AccountIcon className={ styles.icon } color='white' />)
        }
      </IconButton>);

    return (
      <IconMenu
        value={ selected ? renderAccount(selected)(selected) : null }
        onChange={ this.onAccountSelect }
        iconButtonElement={ accountsButton }
        animated={ false }
      >
        { Object.values(all).map(renderAccount(selected)) }
      </IconMenu>
    );
  }

  onAccountSelect = (e, address) => {
    this.props.actions.select(address);
  };
}
