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

import BigNumber from 'bignumber.js';
import moment from 'moment';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { IdentityIcon, IdentityName, Input, InputAddress } from '~/ui';
import ShortenedHash from '~/ui/ShortenedHash';
import { txLink } from '~/3rdparty/etherscan/links';

import styles from '../../contract.css';

export default class Event extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    event: PropTypes.object.isRequired,
    netVersion: PropTypes.string.isRequired
  }

  state = {
    transaction: {}
  }

  componentDidMount () {
    this.retrieveTransaction();
  }

  render () {
    const { event, netVersion } = this.props;
    const { block, transaction } = this.state;

    const classes = `${styles.event} ${styles[event.state]}`;
    const url = txLink(event.transactionHash, false, netVersion);
    const keys = Object.keys(event.params).join(', ');
    const values = Object.keys(event.params).map((name, index) => {
      const param = event.params[name];

      return (
        <div className={ styles.eventValue } key={ `${event.key}_val_${index}` }>
          { this.renderParam(name, param) }
        </div>
      );
    });

    return (
      <tr className={ classes }>
        <td className={ styles.timestamp }>
          <div>{
            event.state === 'pending'
              ? (
                <FormattedMessage
                  id='contract.events.eventPending'
                  defaultMessage='pending'
                />
              )
              : this.formatBlockTimestamp(block)
          }</div>
          <div>{ this.formatNumber(transaction.blockNumber) }</div>
        </td>
        <td className={ styles.txhash }>
          { this.renderAddressName(transaction.from) }
        </td>
        <td className={ styles.txhash }>
          <div className={ styles.eventType }>
            { event.type }({ keys })
          </div>
          <a href={ url } target='_blank'><ShortenedHash data={ event.transactionHash } /></a>
        </td>
        <td className={ styles.eventDetails }>
          <div className={ styles.eventParams }>
            { values }
          </div>
        </td>
      </tr>
    );
  }

  renderAddressName (address, withName = true) {
    return (
      <span className={ styles.eventAddress }>
        <IdentityIcon
          address={ address }
          className={ styles.eventIdentityicon }
          center
          inline
        />
        {
          withName
            ? <IdentityName address={ address } />
            : address
        }
      </span>
    );
  }

  renderParam (name, param) {
    const { api } = this.context;

    switch (param.type) {
      case 'address':
        return (
          <InputAddress
            disabled
            text
            className={ styles.input }
            value={ param.value }
            label={ name }
          />
        );

      default:
        let value;

        if (api.util.isInstanceOf(param.value, BigNumber)) {
          value = param.value.toFormat(0);
        } else if (api.util.isArray(param.value)) {
          value = api.util.bytesToHex(param.value);
        } else {
          value = param.value.toString();
        }

        return (
          <Input
            readOnly
            allowCopy
            className={ styles.input }
            value={ value }
            label={ name }
          />
        );
    }
  }

  formatBlockTimestamp (block) {
    if (!block) {
      return null;
    }

    return moment(block.timestamp).fromNow();
  }

  formatNumber (number) {
    if (!number) {
      return null;
    }

    return new BigNumber(number).toFormat();
  }

  retrieveTransaction () {
    const { api } = this.context;
    const { event } = this.props;

    Promise
      .all([
        api.eth.getBlockByNumber(event.blockNumber),
        api.eth.getTransactionByHash(event.transactionHash)
      ])
      .then(([block, transaction]) => {
        this.setState({ block, transaction });
      })
      .catch((error) => {
        console.warn('retrieveTransaction', error);
      });
  }
}
