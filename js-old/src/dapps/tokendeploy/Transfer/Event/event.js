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

// TODO: This is a copy & paste for Deploy/Event -> render() different. Not very DRY
import moment from 'moment';
import React, { Component, PropTypes } from 'react';

import { api } from '../../parity';
import { txLink } from '../../services';
import IdentityIcon from '../../IdentityIcon';
import styles from '../../Deploy/Event/event.css';

export default class Event extends Component {
  static contextTypes = {
    accounts: PropTypes.object.isRequired,
    registryInstance: PropTypes.object.isRequired,
    tokenregInstance: PropTypes.object.isRequired
  }

  static propTypes = {
    event: PropTypes.object.isRequired,
    token: PropTypes.object.isRequired
  }

  state = {
    block: null
  }

  componentDidMount () {
    this.lookup();
  }

  render () {
    const { event, token } = this.props;
    const { block } = this.state;
    const isPending = event.type === 'pending';

    return (
      <tr className={ isPending ? styles.pending : styles.mined }>
        <td className={ styles.blocknumber }>
          <div>{ (isPending || !block) ? '' : moment(block.timestamp).fromNow() }</div>
          <div>{ isPending ? 'Pending' : event.blockNumber.toFormat() }</div>
        </td>
        <td>{ event.event }</td>
        <td className={ styles.description }>
          <div>{ isPending ? '' : token.coin.tla }</div>
          <div>{ isPending ? '' : token.coin.name }</div>
        </td>
        <td className={ styles.address }>
          { this.renderAddress(event.params.from) }
        </td>
        <td className={ styles.value }>
          <div>{ event.params.value.div(1000000).toFormat(6) }</div>
          <div>⇒</div>
          <div><a href={ txLink(event.transactionHash) } target='_blank' className={ styles.link }>{ this.renderHash(event.transactionHash) }</a></div>
        </td>
        <td className={ styles.address }>
          { this.renderAddress(event.params.to) }
        </td>
      </tr>
    );
  }

  renderAddress (address) {
    const { accounts } = this.context;
    const account = accounts[address];

    return (
      <div>
        <IdentityIcon address={ address } />
        <span>{ account ? account.name : address }</span>
      </div>
    );
  }

  renderHash (hash) {
    return `${hash.substr(0, 10)}...${hash.slice(-10)}`;
  }

  lookup () {
    const { event } = this.props;

    if (event.type === 'pending') {
      return;
    }

    api.parity
      .getBlockHeaderByNumber(event.blockNumber)
      .then((block) => {
        this.setState({ block });
      });
  }
}
