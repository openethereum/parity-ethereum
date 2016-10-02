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

import moment from 'moment';
import React, { Component, PropTypes } from 'react';

import { api } from '../../parity';

import styles from './event.css';

export default class Event extends Component {
  static contextTypes = {
    registryInstance: PropTypes.object.isRequired,
    tokenregInstance: PropTypes.object.isRequired
  }

  static propTypes = {
    event: PropTypes.object.isRequired
  }

  state = {
    block: null,
    coin: {}
  }

  componentDidMount () {
    const { event } = this.props;
    const { registryInstance, tokenregInstance } = this.context;

    if (event.type === 'pending') {
      return;
    }

    const registry = event.params.tokenreg === tokenregInstance.address
      ? tokenregInstance
      : registryInstance;

    Promise
      .all([
        api.eth.getBlockByNumber(event.blockNumber),
        registry.fromAddress.call({}, [event.params.coin])
      ])
      .then(([block, coin]) => {
        const [id, tla, base, name, owner] = coin;

        this.setState({
          block,
          coin: { id, tla, base, name, owner }
        });
      });
  }

  render () {
    const { event } = this.props;
    const { block, coin } = this.state;
    const isPending = event.type === 'pending';

    return (
      <tr className={ isPending ? styles.pending : styles.mined }>
        <td className={ styles.blocknumber }>
          <div>{ (isPending || !block) ? '' : moment(block.timestamp).fromNow() }</div>
          <div>{ isPending ? 'Pending' : event.blockNumber.toFormat() }</div>
        </td>
        <td>{ event.event }</td>
        <td className={ styles.frominfo }>
          <div>{ this.renderAddress(event.params.owner) }</div>
          <div>{ this.renderHash(event.transactionHash) }</div>
        </td>
        <td className={ styles.description }>
          <div>{ isPending ? '' : coin.tla }</div>
          <div>{ isPending ? '' : coin.name }</div>
        </td>
      </tr>
    );
  }

  renderAddress (address) {
    return address;
  }

  renderHash (hash) {
    return `${hash.substr(0, 10)}...${hash.slice(-10)}`;
  }
}
