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

import styles from './dapp.css';

const dapphost = process.env.NODE_ENV === 'production' ? 'http://127.0.0.1:8080/ui' : '';

export default class Dapp extends Component {
  static propTypes = {
    params: PropTypes.object
  };

  render () {
    const { name, type } = this.props.params;
    const src = (type === 'builtin')
      ? `${dapphost}/${name}.html`
      : `http://127.0.0.1:8080/${name}/`;

    return (
      <iframe
        className={ styles.frame }
        frameBorder={ 0 }
        name={ name }
        sandbox='allow-same-origin allow-scripts'
        scrolling='auto'
        src={ src }>
      </iframe>
    );
  }
}
