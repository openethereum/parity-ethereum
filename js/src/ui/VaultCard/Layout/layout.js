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

import Title from '~/ui/Title';
import IdentityIcon from '~/ui/IdentityIcon';

import styles from './layout.css';

export default class Layout extends Component {
  static propTypes = {
    children: PropTypes.node,
    vault: PropTypes.object.isRequired,
    withBorder: PropTypes.bool
  };

  render () {
    const { children, vault, withBorder } = this.props;
    const { isOpen, meta, name } = vault;

    return (
      <div
        className={
          [
            styles.layout,
            withBorder
              ? styles.border
              : null
          ].join(' ')
        }
      >
        <IdentityIcon
          address={ name }
          center
          className={
            [
              styles.identityIcon,
              isOpen || withBorder
                ? styles.unlocked
                : styles.locked
            ].join(' ')
          }
        />
        <div className={ styles.info }>
          <Title
            byline={ meta.description }
            title={ name }
          />
          { children }
        </div>
      </div>
    );
  }
}
