import React, { Component, PropTypes } from 'react';
import { Checkbox, FloatingActionButton, MenuItem } from 'material-ui';

import CommunicationContacts from 'material-ui/svg-icons/communication/contacts';

import AddressSelector from '../../AddressSelector';
import Form, { Input, Select } from '../../../ui/Form';
import IdentityIcon from '../../../ui/IdentityIcon';

import styles from '../style.css';

const CHECK_STYLE = {
  position: 'absolute',
  top: '38px',
  left: '1em'
};

export default class Details extends Component {
  static contextTypes = {
    accounts: PropTypes.array
  }

  static propTypes = {
    address: PropTypes.string,
    all: PropTypes.bool,
    extras: PropTypes.bool,
    recipient: PropTypes.string,
    recipientError: PropTypes.string,
    tag: PropTypes.string,
    total: PropTypes.string,
    totalError: PropTypes.string,
    value: PropTypes.string,
    valueError: PropTypes.string,
    onChange: PropTypes.func.isRequired
  }

  state = {
    showAddresses: false
  }

  render () {
    const label = `amount to transfer (in ${this.props.tag})`;

    return (
      <Form>
        { this.renderTokenSelect() }
        { this.renderAddressSelect() }
        { this.renderAddress() }
        <div className={ styles.columns }>
          <div>
            <Input
              disabled={ this.props.all }
              label={ label }
              hint='the amount to transfer to the recipient'
              value={ this.props.value }
              error={ this.props.valueError }
              onChange={ this.onEditValue } />
          </div>
          <div>
            <Checkbox
              checked={ this.props.all }
              label='full account balance'
              onCheck={ this.onCheckAll }
              style={ CHECK_STYLE } />
          </div>
        </div>
        <div className={ styles.columns }>
          <div>
            <Input
              disabled
              label='total transaction amount'
              hint='the total amount of the transaction'
              error={ this.props.totalError }
              value={ `${this.props.total} ΞTH` } />
          </div>
          <div>
            <Checkbox
              checked={ this.props.extras }
              label='advanced sending options'
              onCheck={ this.onCheckExtras }
              style={ CHECK_STYLE } />
          </div>
        </div>
      </Form>
    );
  }

  renderAddress () {
    const iconClass = this.props.recipientError
      ? `${styles.floatimg} ${styles.grayscale}`
      : styles.floatimg;

    const iconAddress = this.props.recipientError
      ? '0x00'
      : this.props.recipient;

    return (
      <div className={ styles.address }>
        <Input
          label='recipient address'
          hint='the recipient address'
          error={ this.props.recipientError }
          value={ this.props.recipient }
          onChange={ this.onEditRecipient } />
        <div className={ iconClass }>
          <IdentityIcon
            inline center
            address={ iconAddress } />
        </div>
        <div className={ styles.floatbutton }>
          <FloatingActionButton
            mini
            onTouchTap={ this.onContacts }>
            <CommunicationContacts />
          </FloatingActionButton>
        </div>
      </div>
    );
  }

  renderAddressSelect () {
    if (!this.state.showAddresses) {
      return null;
    }

    return (
      <AddressSelector
        onSelect={ this.onSelectRecipient } />
    );
  }

  renderTokenSelect () {
    const account = this.context.accounts.find((acc) => acc.address === this.props.address);
    const items = account.balances.map((balance) => {
      const token = balance.token;
      const label = (
        <div className={ styles.token }>
          <img src={ token.images.small } />
          <div>{ token.name }</div>
        </div>
      );

      return (
        <MenuItem
          key={ token.tag }
          primaryText={ token.name }
          value={ token.tag }
          label={ label }
          leftIcon={ <img src={ token.images.small } /> } />
      );
    });

    return (
      <Select
        label='type of transfer'
        hint='type of token to transfer'
        value={ this.props.tag }
        onChange={ this.onChangeToken }>
        { items }
      </Select>
    );
  }

  onChangeToken = (event, value) => {
    const account = this.context.accounts.find((acc) => acc.address === this.props.address);
    this.props.onChange('tag', account.balances[value].token.tag);
  }

  onSelectRecipient = (recipient) => {
    this.setState({ showAddresses: false });
    this.props.onChange('recipient', recipient);
  }

  onEditRecipient = (event) => {
    this.onSelectRecipient(event.target.value);
  }

  onEditValue = (event) => {
    this.props.onChange('value', event.target.value);
  }

  onCheckAll = () => {
    this.props.onChange('all', !this.props.all);
  }

  onCheckExtras = () => {
    this.props.onChange('extras', !this.props.extras);
  }

  onContacts = () => {
    this.setState({
      showAddresses: true
    });
  }
}
