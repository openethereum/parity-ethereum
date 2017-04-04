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

import Response from '../Response';
import styles from './Call.css';

export default class Call extends Component {
  render () {
    let { callNo, name, params, response } = this.props.call;

    params = this.formatParams(params);
    return (
      <div
        onMouseEnter={ this.setActiveCall }
        ref={ this.setElement }
        className={ styles.call }
        { ...this._test(`call-${callNo}`) }
      >
        <span className={ styles.callNo } { ...this._test('callNo') }>#{ callNo }</span>
        <pre { ...this._test('name') }>{ name }({ params })</pre>
        <Response response={ response } />
      </div>
    );
  }

  setElement = el => {
    this.element = el;
  }

  setActiveCall = () => {
    this.props.setActiveCall(this.props.call, this.element);
  }

  formatParams (params) {
    return params.reduce((str, p) => {
      if (str !== '') {
        str += ', ';
      }
      if (p === undefined) {
        return str;
      }
      if (typeof p === 'object' || typeof p === 'string') {
        p = JSON.stringify(p);
      }
      return str + p;
    }, '');
  }

  static propTypes = {
    call: PropTypes.object.isRequired,
    setActiveCall: PropTypes.func.isRequired
  }
}
