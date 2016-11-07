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

import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import { Checkbox, MenuItem } from 'material-ui';

import Form, { Input, InputAddressSelect, Select } from '../../../ui/Form';

import imageUnknown from '../../../../assets/images/contracts/unknown-64x64.png';
import styles from '../transfer.css';

const CHECK_STYLE = {
  position: 'absolute',
  top: '38px',
  left: '1em'
};

export default class Details extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    address: PropTypes.string,
    balance: PropTypes.object,
    all: PropTypes.bool,
    extras: PropTypes.bool,
    images: PropTypes.object.isRequired,
    recipient: PropTypes.string,
    recipientError: PropTypes.string,
    tag: PropTypes.string,
    total: PropTypes.string,
    totalError: PropTypes.string,
    value: PropTypes.string,
    valueError: PropTypes.string,
    onChange: PropTypes.func.isRequired
  }

  render () {
    const { all, extras, tag, total, totalError, value, valueError } = this.props;
    const label = `amount to transfer (in ${tag})`;

    return (
      <Form>
        { this.renderTokenSelect() }
        { this.renderToAddress() }
        <div className={ styles.columns }>
          <div>
            <Input
              disabled={ all }
              label={ label }
              hint='the amount to transfer to the recipient'
              value={ value }
              error={ valueError }
              onChange={ this.onEditValue } />
          </div>
          <div>
            <Checkbox
              checked={ all }
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
              error={ totalError }>
              <div className={ styles.inputoverride }>
                { total }<small> ETH</small>
              </div>
            </Input>
          </div>
          <div>
            <Checkbox
              checked={ extras }
              label='advanced sending options'
              onCheck={ this.onCheckExtras }
              style={ CHECK_STYLE } />
          </div>
        </div>
      </Form>
    );
  }

  renderToAddress () {
    const { recipient, recipientError } = this.props;

    return (
      <div className={ styles.address }>
        <InputAddressSelect
          label='recipient address'
          hint='the recipient address'
          error={ recipientError }
          value={ recipient }
          onChange={ this.onEditRecipient } />
      </div>
    );
  }

  renderTokenSelect () {
    const { api } = this.context;
    const { balance, images, tag } = this.props;

    const items = balance.tokens
      .filter((token, index) => !index || token.value.gt(0))
      .map((balance, index) => {
        const token = balance.token;
        const isEth = index === 0;
        const imagesrc = token.image || images[token.address] || imageUnknown;
        let value = 0;

        if (isEth) {
          value = api.util.fromWei(balance.value).toFormat(3);
        } else {
          const format = balance.token.format || 1;
          const decimals = format === 1 ? 0 : Math.min(3, Math.floor(format / 10));
          value = new BigNumber(balance.value).div(format).toFormat(decimals);
        }

        const label = (
          <div className={ styles.token }>
            <img src={ imagesrc } />
            <div className={ styles.tokenname }>
              { token.name }
            </div>
            <div className={ styles.tokenbalance }>
              { value }<small> { token.tag }</small>
            </div>
          </div>
        );

        return (
          <MenuItem
            key={ token.tag }
            value={ token.tag }
            label={ label }>
            { label }
          </MenuItem>
        );
      });

    return (
      <Select
        className={ styles.tokenSelect }
        label='type of token transfer'
        hint='type of token to transfer'
        value={ tag }
        onChange={ this.onChangeToken }>
        { items }
      </Select>
    );
  }

  onChangeToken = (event, index, tag) => {
    this.props.onChange('tag', tag);
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
