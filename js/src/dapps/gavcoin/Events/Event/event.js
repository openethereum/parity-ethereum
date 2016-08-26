import React, { Component, PropTypes } from 'react';

import { formatBlockNumber, formatCoins, formatEth } from '../../format';

const { IdentityIcon } = window.parity.react;

const EMPTY_COLUMN = (
  <td></td>
);

export default class Event extends Component {
  static contextTypes = {
    accounts: PropTypes.array
  }

  static propTypes = {
    event: PropTypes.object,
    value: PropTypes.object,
    price: PropTypes.object,
    fromAddress: PropTypes.string,
    toAddress: PropTypes.string
  }

  render () {
    const { event, fromAddress, toAddress, price, value } = this.props;
    const { blockNumber, state, type } = event;
    const cls = `event ${state} ${type.toLowerCase()}`;

    return (
      <tr className={ cls }>
        { this.renderBlockNumber(blockNumber) }
        { this.renderType(type) }
        { this.renderValue(value) }
        { this.renderPrice(price) }
        { this.renderAddress(fromAddress) }
        { this.renderAddress(toAddress) }
      </tr>
    );
  }

  renderBlockNumber (blockNumber) {
    return (
      <td className='blocknumber'>
        { formatBlockNumber(blockNumber) }
      </td>
    );
  }

  renderAddress (address) {
    if (!address) {
      return EMPTY_COLUMN;
    }

    return (
      <td className='account'>
        <IdentityIcon inline center address={ address } />
        { this.renderAddressName(address) }
      </td>
    );
  }

  renderAddressName (address) {
    const account = this.context.accounts.find((_account) => _account.address === address);

    if (account) {
      return (
        <div className='name'>
          { account.name }
        </div>
      );
    }

    return (
      <div className='address'>
        { address }
      </div>
    );
  }

  renderPrice (price) {
    if (!price) {
      return EMPTY_COLUMN;
    }

    return (
      <td className='ethvalue'>
        { formatEth(price) }ΞTH
      </td>
    );
  }

  renderValue (value) {
    if (!value) {
      return EMPTY_COLUMN;
    }

    return (
      <td className='gavvalue'>
        { formatCoins(value) }GAV
      </td>
    );
  }

  renderType (type) {
    return (
      <td className='type'>
        { type }
      </td>
    );
  }
}
