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

import keycode from 'keycode';
import React, { Component, PropTypes } from 'react';
import ReactDOM from 'react-dom';

import styles from './modal.css';

export default class Modal extends Component {
  static propTypes = {
    buttons: PropTypes.node,
    children: PropTypes.node,
    header: PropTypes.node,
    secondary: PropTypes.bool,
    onClose: PropTypes.func.isRequired
  };

  static defaultProps = {
    secondary: false
  };

  render () {
    const { children, buttons, header, secondary } = this.props;

    const modalClasses = [ styles.modal ];

    if (secondary) {
      modalClasses.push(styles.secondary);
    }

    return (
      <div
        className={ modalClasses.join(' ') }
        onClick={ this.handleClose }
        onKeyUp={ this.handleKeyPress }
      >
        <div
          className={ styles.dialog }
          onClick={ this.stopEvent }
          ref={ this.handleSetRef }
          tabIndex={ open ? 0 : null }
        >
          <div className={ styles.header }>
            { header }
            <div
              className={ styles.close }
              onClick={ this.handleClose }
              onKeyPress={ this.handleCloseKeyPress }
              tabIndex={ open ? 0 : null }
              title='close'
            />
          </div>

          <div className={ styles.content }>
            { children }
          </div>

          {
            buttons
            ? (
              <div className={ styles.footer }>
                { buttons }
              </div>
            )
            : null
          }
        </div>
      </div>
    );
  }

  stopEvent = (event) => {
    event.stopPropagation();
    event.preventDefault();

    return false;
  }

  handleKeyPress = (event) => {
    const codeName = keycode(event);

    if (codeName === 'esc') {
      return this.handleClose();
    }

    return event;
  }

  handleCloseKeyPress = (event) => {
    const codeName = keycode(event);

    if (codeName === 'enter') {
      return this.handleClose();
    }

    return event;
  }

  handleSetRef = (containerRef) => {
    // Focus after the modal is open
    setTimeout(() => {
      const element = ReactDOM.findDOMNode(containerRef);

      element && element.focus();
    }, 100);
  }

  handleClose = () => {
    this.props.onClose();
  }
}
