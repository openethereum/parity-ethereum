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

import { IdentityIcon } from '~/ui';

import styles from '../vaults.css';

export default class NameLayout extends Component {
  static propTypes = {
    isOpen: PropTypes.bool.isRequired,
    name: PropTypes.string.isRequired
  };

  render () {
    const { isOpen, name } = this.props;

    return (
      <div className={ styles.namebox }>
        <IdentityIcon
          address={ name }
          center
          className={
            [
              styles.identityIcon,
              isOpen
                ? styles.unlocked
                : styles.locked
            ].join(' ')
          }
        />,
        <div className={ styles.name }>
          { name }
        </div>
      </div>
    );
  }
}
