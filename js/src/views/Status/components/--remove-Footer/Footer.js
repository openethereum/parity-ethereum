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
import IconButton from 'material-ui/IconButton';
import IconEventNote from 'material-ui/svg-icons/notification/event-note';

import styles from './Footer.css';

export default class Footer extends Component {

  render () {
    return (
      <footer { ...this._testInherit() }>
        <div className={ styles.footer }>
          <a href='http://ethcore.io'>ethcore.io</a>
          { this.renderLogIcon() }
          <span className={ styles.right }>
            Powered by: { this.props.version }
          </span>
        </div>
      </footer>
    );
  }

  renderLogIcon () {
    const { updateLogging, logging } = this.props;
    const isOffClass = !logging ? styles.off : '';

    const onClick = () => updateLogging(!logging);

    return (
      <IconButton
        { ...this._testInherit('log-button') }
        onClick={ onClick }
        tooltip='Toggle logging' tooltipPosition='top-left'
        className={ styles.logButton }
        >
        <IconEventNote className={ `${styles.logIcon} ${isOffClass}` } />
      </IconButton>
    );
  }

  static propTypes = {
    version: PropTypes.string.isRequired,
    logging: PropTypes.bool.isRequired,
    updateLogging: PropTypes.func.isRequired
  }

}
