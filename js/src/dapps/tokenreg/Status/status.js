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

import Chip from '../Chip';

import styles from './status.css';

const { api } = window.parity;

export default class Status extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    fee: PropTypes.object.isRequired
  };

  render () {
    const { fee } = this.props;

    return (
      <div className={ styles.status }>
        <h1 className={ styles.title }>Token Registry</h1>
        <h3 className={ styles.byline }>A global registry of all recognised tokens on the network</h3>
        <Chip
          isAddress={ false }
          value={ api.util.fromWei(fee).toFixed(3) + 'ETH' }
          label='Fee'
        />
      </div>
    );
  }
}
