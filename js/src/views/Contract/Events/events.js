// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import { uniq } from 'lodash';

import { Container, Loading } from '~/ui';

import Event from './Event';
import styles from '../contract.css';

export default class Events extends Component {
  static contextTypes = {
    api: PropTypes.object
  };

  static propTypes = {
    isTest: PropTypes.bool.isRequired,
    isLoading: PropTypes.bool,
    events: PropTypes.array
  };

  static defaultProps = {
    isLoading: false,
    events: []
  };

  render () {
    const { events, isTest, isLoading } = this.props;

    if (isLoading) {
      return (
        <Container title='events'>
          <div>
            <Loading size={ 2 } />
          </div>
        </Container>
      );
    }

    if (!events || !events.length) {
      return (
        <Container title='events'>
          <p>No events has been sent from this contract.</p>
        </Container>
      );
    }

    const eventsKey = uniq(events.map((e) => e.key));
    const list = eventsKey.map((eventKey) => {
      const event = events.find((e) => e.key === eventKey);

      return (
        <Event
          key={ event.key }
          event={ event }
          isTest={ isTest }
        />
      );
    });

    return (
      <Container title='events'>
        <table className={ styles.events }>
          <thead>
            <tr>
              <th />
              <th className={ styles.origin }>
                origin
              </th>
            </tr>
          </thead>
          <tbody>{ list }</tbody>
        </table>
      </Container>
    );
  }
}
