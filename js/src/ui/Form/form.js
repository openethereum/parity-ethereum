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

import styles from './form.css';

export default class Form extends Component {
  static propTypes = {
    children: PropTypes.node,
    className: PropTypes.string
  }

  render () {
    const { className } = this.props;
    const classes = [ styles.form ];

    if (className) {
      classes.push(className);
    }

    // HACK: hidden inputs to disable Chrome's autocomplete
    return (
      <form
        autoComplete='new-password'
        className={ classes.join(' ') }
      >
        <div className={ styles.autofill }>
          <input type='text' name='fakeusernameremembered' />
          <input type='password' name='fakepasswordremembered' />
        </div>
        { this.props.children }
      </form>
    );
  }
}
