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

import IdentityIcon from '~/ui/IdentityIcon';

import styles from './vaultTag.css';

export default class VaultTag extends Component {
  static propTypes = {
    vault: PropTypes.string.isRequired
  };

  render () {
    const { vault } = this.props;

    return (
      <div className={ styles.vault }>
        <div className={ styles.vaultBody }>
          <IdentityIcon
            address={ vault }
            inline
          />
          <div className={ styles.text }>
            { vault }
          </div>
        </div>
      </div>
    );
  }
}
