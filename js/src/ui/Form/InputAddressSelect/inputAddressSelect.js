import React, { Component, PropTypes } from 'react';
import { MenuItem, Toggle } from 'material-ui';

import IdentityIcon from '../../IdentityIcon';
import InputAddress from '../InputAddress';
import Select from '../Select';

import styles from './inputAddressSelect.css';

export default class InputAddressSelect extends Component {
  static contextTypes = {
    accounts: PropTypes.array.isRequired,
    contacts: PropTypes.array.isRequired
  };

  static propTypes = {
    disabled: PropTypes.bool,
    error: PropTypes.string,
    label: PropTypes.string,
    hint: PropTypes.string,
    value: PropTypes.string,
    onChange: PropTypes.func
  };

  state = {
    editing: false,
    entries: []
  }

  render () {
    const { editing } = this.state;

    return (
      <div className={ styles.inputselect }>
        { editing ? this.renderInput() : this.renderSelect() }
        <Toggle
          className={ styles.toggle }
          label='Edit'
          labelPosition='right'
          toggled={ editing }
          onToggle={ this.onToggle } />
      </div>
    );
  }

  renderInput () {
    const { disabled, error, hint, label, value } = this.props;

    return (
      <InputAddress
        disabled={ disabled }
        error={ error }
        hint={ hint }
        label={ label }
        value={ value }
        onChange={ this.onChangeInput } />
    );
  }

  renderSelect () {
    const { disabled, error, hint, label, value } = this.props;

    return (
      <Select
        disabled={ disabled }
        label={ label }
        hint={ hint }
        error={ error }
        value={ value }
        onChange={ this.onChangeSelect }>
        { this.renderSelectAccounts() }
      </Select>
    );
  }

  renderAccountItem = (account) => {
    const item = (
      <div className={ styles.account }>
        <div className={ styles.image }>
          <IdentityIcon
            inline center
            address={ account.address } />
        </div>
        <div className={ styles.details }>
          <div className={ styles.name }>
            { account.name || 'Unnamed' }
          </div>
        </div>
      </div>
    );

    return (
      <MenuItem
        key={ account.address }
        value={ account.address }
        label={ item }>
        { item }
      </MenuItem>
    );
  }

  renderSelectAccounts () {
    const { accounts, contacts } = this.context;

    const entriesAccounts = accounts.map(this.renderAccountItem);
    const entriesContacts = contacts.map(this.renderAccountItem);

    return entriesAccounts.concat(entriesContacts);
  }

  onToggle = () => {
    const { editing } = this.state;

    this.setState({
      editing: !editing
    });
  }

  onChangeInput = (event, value) => {
    this.props.onChange(event, value);
  }

  onChangeSelect = (event, idx) => {
    const { accounts, contacts } = this.context;
    const entries = accounts.concat(contacts);
    console.log(entries[idx].address);

    this.props.onChange(event, entries[idx].address);
  }
}
