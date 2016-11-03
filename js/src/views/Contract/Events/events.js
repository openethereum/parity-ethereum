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

import { Container, ContainerTitle } from '../../../ui';

import Event from './Event';
import styles from '../contract.css';

export default class Events extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    events: PropTypes.array,
    isTest: PropTypes.bool.isRequired
  }

  render () {
    const { events, isTest } = this.props;

    if (!events || !events.length) {
      return null;
    }

    const list = events.map((event) => {
      return (
        <Event
          key={ event.key }
          event={ event }
          isTest={ isTest } />
      );
    });

    return (
      <Container>
        <ContainerTitle title='events' />
        <table className={ styles.events }>
          <tbody>{ list }</tbody>
        </table>
      </Container>
    );
  }
}
