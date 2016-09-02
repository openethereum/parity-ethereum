import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import { Checkbox, MenuItem } from 'material-ui';

import Api from '../../../api';
import Form, { Input, InputAddressSelect, Select } from '../../../ui/Form';

import styles from '../style.css';

const CHECK_STYLE = {
  position: 'absolute',
  top: '38px',
  left: '1em'
};

export default class Details extends Component {
  static contextTypes = {
    accounts: PropTypes.array.isRequired
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
                { total }<small> ΞTH</small>
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
    const { accounts } = this.context;
    const { address, tag } = this.props;

    const account = accounts.find((acc) => acc.address === address);
    const items = account.balances.map((balance, idx) => {
      const token = balance.token;
      const isEth = idx === 0;
      let value = 0;

      if (isEth) {
        value = Api.format.fromWei(balance.value).toFormat(3);
      } else {
        value = new BigNumber(balance.value).div(balance.token.format || 1).toFormat(3);
      }

      const label = (
        <div className={ styles.token }>
          <img src={ token.images.small } />
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
        label='type of transfer'
        hint='type of token to transfer'
        value={ tag }
        onChange={ this.onChangeToken }>
        { items }
      </Select>
    );
  }

  onChangeToken = (event, value) => {
    const { accounts } = this.context;
    const { address } = this.props;

    const account = accounts.find((acc) => acc.address === address);

    this.props.onChange('tag', account.balances[value].token.tag);
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
