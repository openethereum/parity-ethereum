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

import styles from './shortenedHash.css';

export default class ShortenedHash extends Component {
  static propTypes = {
    data: PropTypes.string.isRequired
  }

  render () {
    const { data } = this.props;

    let shortened = data.toLowerCase();

    if (shortened.slice(0, 2) === '0x') {
      shortened = shortened.slice(2);
    }
    if (shortened.length > (6 + 6)) {
      shortened = shortened.slice(0, 6) + '…' + shortened.slice(-6);
    }

    return (
      <abbr className={ styles.hash } title={ data }>{ shortened }</abbr>
    );
  }
}
