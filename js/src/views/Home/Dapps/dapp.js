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

import moment from 'moment';
import React, { Component, PropTypes } from 'react';
import { Link } from 'react-router';

import { DappIcon } from '~/ui';

import styles from '../home.css';

export default class Dapp extends Component {
  static propTypes = {
    id: PropTypes.string.isRequired,
    store: PropTypes.object.isRequired,
    timestamp: PropTypes.number.isRequired
  }

  state = {
    dapp: null
  }

  componentWillMount () {
    this.loadApp();
  }

  render () {
    const { id, timestamp } = this.props;
    const { dapp } = this.state;

    if (!dapp) {
      return null;
    }

    return (
      <tr>
        <td className={ styles.timestamp }>
          { moment(timestamp).fromNow() }
        </td>
        <td className={ styles.entry }>
          <Link to={ `/app/${id}` }>
            <DappIcon
              app={ dapp }
              className={ styles.dappIcon }
              small
            />
            <span>
              { dapp.name }
            </span>
          </Link>
        </td>
      </tr>
    );
  }

  loadApp = () => {
    const { id, store } = this.props;

    return store
      .loadApp(id)
      .then((dapp) => {
        console.log(id, dapp);
        this.setState({ dapp });
      })
      .catch((error) => {
        console.warn(`Unable to load ${id}`, error);
      });
  }
}
