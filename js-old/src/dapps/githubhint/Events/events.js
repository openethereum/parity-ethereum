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
import moment from 'moment';

import styles from './events.css';

export default class Events extends Component {
  static propTypes = {
    eventIds: PropTypes.array.isRequired,
    events: PropTypes.array.isRequired
  }

  render () {
    return (
      <table className={ styles.list }>
        <tbody>
          { this.props.eventIds.map((id) => this.renderEvent(id, this.props.events[id])) }
        </tbody>
      </table>
    );
  }

  renderEvent = (eventId, event) => {
    return (
      <tr key={ `event_${eventId}` } data-busy={ event.registerBusy } data-error={ event.registerError }>
        <td>
          <div>{ moment(event.timestamp).fromNow() }</div>
          <div>{ event.registerState }</div>
        </td>
        <td>
          <div>{ event.contentUrl || `${event.contentRepo}/${event.contentCommit}` }</div>
          <div>{ event.contentHash }</div>
        </td>
      </tr>
    );
  }
}
