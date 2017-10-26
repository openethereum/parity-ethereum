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

import React, { Component, PropTypes } from 'react';

import { formatBlockNumber, formatBlockTimestamp, formatSignature } from '../format';
import { attachEvents } from '../services';
import IdentityIcon from '../IdentityIcon';

import styles from './events.css';

export default class Events extends Component {
  static propTypes = {
    accountsInfo: PropTypes.object.isRequired,
    contract: PropTypes.object.isRequired
  }

  state = {
    events: []
  }

  componentDidMount () {
    const { contract } = this.props;

    attachEvents(contract, (state) => {
      this.setState(state);
    });
  }

  render () {
    const { events } = this.state;

    if (!events.length) {
      return null;
    }

    return (
      <div className={ styles.events }>
        <table>
          <tbody>
            { this.renderEvents() }
          </tbody>
        </table>
      </div>
    );
  }

  renderEvents () {
    const { accountsInfo } = this.props;
    const { events } = this.state;

    return events.map((event) => {
      const name = accountsInfo[event.params.creator]
        ? accountsInfo[event.params.creator].name
        : event.params.creator;

      return (
        <tr className={ styles[event.state] } key={ event.key }>
          <td className={ styles.timestamp }>{ formatBlockTimestamp(event.block) }</td>
          <td className={ styles.blockNumber }>{ formatBlockNumber(event.blockNumber) }</td>
          <td className={ styles.owner }>
            <IdentityIcon address={ event.params.creator } />
            <div>{ name }</div>
          </td>
          <td className={ styles.signature }>{ formatSignature(event.params.signature) }</td>
          <td className={ styles.methodName }>{ event.params.method }</td>
        </tr>
      );
    });
  }
}
