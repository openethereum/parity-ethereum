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

import Button from '../Button';

import styles from './modal.css';
import CloseImage from '~/../assets/images/dapps/close.svg';

export default class Modal extends Component {
  static propTypes = {
    actions: PropTypes.array,
    children: PropTypes.node,
    header: PropTypes.node,
    secondary: PropTypes.bool,
    onClose: PropTypes.func.isRequired,
    onConfirm: PropTypes.func
  };

  static defaultProps = {
    actions: null,
    secondary: false
  };

  render () {
    const { children, actions, header, secondary } = this.props;

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
            >
              <img
                className={ styles.closeIcon }
                src={ CloseImage }
              />
            </div>
          </div>

          <div className={ styles.content }>
            { children }
          </div>

          { actions ? this.renderActions(actions) : null }
        </div>
      </div>
    );
  }

  renderActions (actions) {
    return (
      <div className={ styles.footer }>
        { actions.map((action) => {
          let onClick = () => {};

          switch (action.type) {
            case 'confirm':
              onClick = this.handleConfirm;
              break;

            case 'close':
              onClick = this.handleClose;
              break;
          }

          return (
            <Button
              key={ action.type }
              label={ action.label }
              warning={ action.warning }
              onClick={ onClick }
            />
          );
        }) }
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

  handleConfirm = () => {
    this.props.onConfirm && this.props.onConfirm();
  }

  handleClose = () => {
    this.props.onClose();
  }
}
