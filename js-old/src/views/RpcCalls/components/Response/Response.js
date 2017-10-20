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
import { isArray, isPlainObject } from 'lodash';

import styles from './Response.css';

export default class Response extends Component {
  render () {
    let { response } = this.props;
    let formatted;

    if (isArray(response)) {
      formatted = this.renderArray();
    }
    if (isPlainObject(response)) {
      formatted = this.renderObject();
    }

    return <pre className={ styles.response }>{ formatted || response }</pre>;
  }

  renderArray () {
    let { response } = this.props;

    return response.map((r, idx) => (
      <span key={ idx }>
        { idx === 0 ? '[' : ',' }
        { idx === 0 ? '' : <br /> }
        { r }
        { idx === response.length - 1 ? ']' : '' }
      </span>
    ));
  }

  renderObject () {
    let { response } = this.props;
    const arr = JSON.stringify(response, null, 1).split('\n');

    return arr.map((any, idx) => (
      <span key={ idx }>
        { any }
        { idx !== 0 && idx !== arr.length - 1 ? <br /> : '' }
      </span>
    ));
  }

  static propTypes = {
    response: PropTypes.any.isRequired
  }
}
