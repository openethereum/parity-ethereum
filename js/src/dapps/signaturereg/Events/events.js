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

import React, { Component, PropTypes } from 'react';

import { formatBlockNumber, formatBlockTimestamp, formatSignature } from '../format';
import { attachEvents } from '../services';

import styles from './events.css';

export default class Events extends Component {
  static propTypes = {
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
    const { events } = this.state;

    return events.map((event) => {
      return (
        <tr className={ styles[event.state] } key={ event.key }>
          <td className={ styles.right }>{ formatBlockNumber(event.blockNumber) }</td>
          <td className={ styles.right }>{ formatBlockTimestamp(event.block) }</td>
          <td>{ event.params.owner }</td>
          <td>{ formatSignature(event.params.signature) }</td>
          <td className={ styles.highlight }>{ event.params.method }</td>
        </tr>
      );
    });
  }
}
