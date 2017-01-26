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

import styles from './modal.css';

export default class Modal extends Component {
  static propTypes = {
    buttons: PropTypes.node,
    children: PropTypes.node,
    error: PropTypes.object,
    header: PropTypes.string
  }

  render () {
    const { children, buttons, error, header } = this.props;

    return (
      <div className={ styles.modal }>
        <div className={ styles.overlay } />
        <div className={ styles.body }>
          <div className={ styles.dialog }>
            <div className={ `${styles.header} ${error ? styles.error : ''}` }>
              { header }
            </div>
            <div className={ styles.content }>
              { error ? this.renderError() : children }
            </div>
            <div className={ styles.footer }>
              { buttons }
            </div>
          </div>
        </div>
      </div>
    );
  }

  renderError () {
    const { error } = this.props;

    return (
      <div>
        <div className={ styles.section }>
          Your operation failed to complete sucessfully. The following error was returned:
        </div>
        <div className={ `${styles.section} ${styles.error}` }>
          { error.toString() }
        </div>
      </div>
    );
  }
}
