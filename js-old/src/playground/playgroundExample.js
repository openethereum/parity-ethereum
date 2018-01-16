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
import reactElementToJSXString from 'react-element-to-jsx-string';

import styles from './playground.css';

export default class PlaygroundExample extends Component {
  static propTypes = {
    children: PropTypes.node,
    name: PropTypes.string
  };

  render () {
    const { children, name } = this.props;

    return (
      <div className={ styles.exampleContainer }>
        { this.renderName(name) }
        <div className={ styles.example }>
          <div className={ styles.code }>
            <code>{ reactElementToJSXString(children) }</code>
          </div>
          <div className={ styles.component }>
            { children }
          </div>
        </div>
      </div>
    );
  }

  renderName (name) {
    if (!name) {
      return null;
    }

    return (
      <p>{ name }</p>
    );
  }
}
