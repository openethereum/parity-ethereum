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

import { Checkbox } from 'material-ui';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import Form, { Input, InputAddressSelect, AddressSelect } from '~/ui/Form';
import { nullableProptype } from '~/util/proptypes';

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
        defaultMessage='amount to transfer (in {tag})'
        values={ {
          tag: token.tag
        } }
      />
    );

    return (
      <Form>
        { this.renderTokenSelect() }
        { this.renderFromAddress() }
        { this.renderToAddress() }
        <div className={ styles.columns }>
          <div>
            <Input
              disabled={ all }
              label={ label }
              hint={
                <FormattedMessage
                  id='transfer.details.amount.hint'
                  defaultMessage='the amount to transfer to the recipient'
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
                  defaultMessage='full account balance'
                />
              }
              onCheck={ this.onCheckAll }
              style={ CHECK_STYLE }
            />
          </div>
        </div>
        <div className={ styles.columns }>
          <div>
            <Input
              disabled
              label={
                <FormattedMessage
                  id='transfer.details.total.label'
                  defaultMessage='total transaction amount'
                />
              }
              error={ totalError }
            >
              <div className={ styles.inputoverride }>
                { total }<small> ETH</small>
              </div>
            </Input>
          </div>

          <div>
            <Checkbox
              checked={ extras }
              label={
                <FormattedMessage
                  id='transfer.details.advanced.label'
                  defaultMessage='advanced sending options'
                />
              }
              onCheck={ this.onCheckExtras }
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
              defaultMessage='sender address'
            />
          }
          hint={
            <FormattedMessage
              id='transfer.details.sender.hint'
              defaultMessage='the sender address'
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
          label={
            <FormattedMessage
              id='transfer.details.recipient.label'
              defaultMessage='recipient address'
            />
          }
          hint={
            <FormattedMessage
              id='transfer.details.recipient.hint'
              defaultMessage='the recipient address'
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

  onChangeToken = (event, index, tokenId) => {
    this.props.onChange('token', tokenId);
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
