// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import { nullableProptype } from '@parity/shared/util/proptypes';
import Form, { AddressSelect, Checkbox, Input, InputAddressSelect, Label } from '@parity/ui/Form';

import TokenSelect from './tokenSelect';
import styles from '../transfer.css';

export const CHECK_STYLE = {
  position: 'absolute',
  top: '38px',
  left: '1em'
};

export default class Details extends Component {
  static propTypes = {
    address: PropTypes.string,
    balance: PropTypes.object,
    all: PropTypes.bool,
    extras: PropTypes.bool,
    sender: PropTypes.string,
    senderError: PropTypes.string,
    recipient: PropTypes.string,
    recipientError: PropTypes.string,
    token: PropTypes.object,
    total: PropTypes.string,
    totalError: PropTypes.string,
    value: PropTypes.string,
    valueError: PropTypes.string,
    onChange: PropTypes.func.isRequired,
    wallet: PropTypes.object,
    senders: nullableProptype(PropTypes.object)
  };

  static defaultProps = {
    wallet: null,
    senders: null
  };

  render () {
    const { all, extras, token, total, totalError, value, valueError } = this.props;
    const label = (
      <FormattedMessage
        id='transfer.details.amount.label'
        defaultMessage='Amount to transfer (in {tag})'
        values={ {
          tag: token.tag
        } }
      />
    );

    let totalAmountStyle = { color: 'rgba(0,0,0,.87)' };

    if (totalError) {
      totalAmountStyle = { color: '#9F3A38' };
    }

    return (
      <Form>
        { this.renderTokenSelect() }
        { this.renderFromAddress() }
        { this.renderToAddress() }
        <div className={ styles.columns }>
          <div>
            <Input
              className={ styles.inputContainer }
              disabled={ all }
              label={ label }
              hint={
                <FormattedMessage
                  id='transfer.details.amount.hint'
                  defaultMessage='The amount to transfer to the recipient'
                />
              }
              value={ value }
              error={ valueError }
              onChange={ this.onEditValue }
            />
          </div>
          <div>
            <Checkbox
              checked={ all }
              label={
                <FormattedMessage
                  id='transfer.details.fullBalance.label'
                  defaultMessage='Full account balance'
                />
              }
              onClick={ this.onCheckAll }
              style={ CHECK_STYLE }
            />
          </div>
        </div>
        <div className={ styles.columns }>
          <div className={ styles.totalTx }>
            <Label className={ styles.transferLabel }>
              <FormattedMessage
                id='transfer.details.total.label'
                defaultMessage='Total transaction amount'
              />
            </Label>
            <div className={ styles.totalAmount } style={ totalAmountStyle }>
              <div>{ total }<small> ETH</small></div>
              <div>{ totalError }</div>
            </div>
          </div>

          <div>
            <Checkbox
              checked={ extras }
              label={
                <FormattedMessage
                  id='transfer.details.advanced.label'
                  defaultMessage='Advanced sending options'
                />
              }
              onClick={ this.onCheckExtras }
              style={ CHECK_STYLE }
            />
          </div>
        </div>
      </Form>
    );
  }

  renderFromAddress () {
    const { sender, senderError, senders } = this.props;

    if (!senders) {
      return null;
    }

    return (
      <div className={ styles.address }>
        <AddressSelect
          accounts={ senders }
          error={ senderError }
          label={
            <FormattedMessage
              id='transfer.details.sender.label'
              defaultMessage='Sender address'
            />
          }
          hint={
            <FormattedMessage
              id='transfer.details.sender.hint'
              defaultMessage='The sender address'
            />
          }
          value={ sender }
          onChange={ this.onEditSender }
        />
      </div>
    );
  }

  renderToAddress () {
    const { recipient, recipientError } = this.props;

    return (
      <div className={ styles.address }>
        <InputAddressSelect
          className={ styles.inputContainer }
          label={
            <FormattedMessage
              id='transfer.details.recipient.label'
              defaultMessage='Recipient address'
            />
          }
          hint={
            <FormattedMessage
              id='transfer.details.recipient.hint'
              defaultMessage='The recipient address'
            />
          }
          error={ recipientError }
          value={ recipient }
          onChange={ this.onEditRecipient }
        />
      </div>
    );
  }

  renderTokenSelect () {
    const { balance, token } = this.props;

    return (
      <TokenSelect
        balance={ balance }
        onChange={ this.onChangeToken }
        value={ token.id }
      />
    );
  }

  onChangeToken = (event, token) => {
    this.props.onChange('token', token);
  }

  onEditSender = (event, sender) => {
    this.props.onChange('sender', sender);
  }

  onEditRecipient = (event, recipient) => {
    this.props.onChange('recipient', recipient);
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
