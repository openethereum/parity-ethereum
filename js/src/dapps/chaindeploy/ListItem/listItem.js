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

import styles from './listItem.css';

export default class ListItem extends Component {
  static propTypes = {
    children: PropTypes.node.isRequired,
    disabled: PropTypes.bool,
    status: PropTypes.string
  }

  render () {
    const { children, disabled } = this.props;

    return (
      <div
        className={
          [
            styles.listItem,
            disabled
              ? styles.muted
              : ''
          ].join(' ')
        }
      >
        <div className={ styles.body }>
          { children }
        </div>
        { this.renderStatus() }
      </div>
    );
  }

  renderStatus () {
    const { status } = this.props;

    if (!status) {
      return null;
    }

    return (
      <div className={ styles.status }>
        { status }
      </div>
    );
  }
}
