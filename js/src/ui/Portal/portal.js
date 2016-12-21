// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import ReactDOM from 'react-dom';
import Portal from 'react-portal';
import keycode from 'keycode';
import { noop } from 'lodash';

import { CloseIcon } from '~/ui/Icons';
import ParityBackground from '~/ui/ParityBackground';

import styles from './portal.css';

export default class Protal extends Component {

  static propTypes = {
    onClose: PropTypes.func.isRequired,

    children: PropTypes.node,
    className: PropTypes.string,
    onKeyDown: PropTypes.func,
    target: PropTypes.any
  };

  state = {
    expanded: false,
    left: 0,
    top: 0
  }

  componentWillMount () {
    this.setPosition({}, () => {
      this.setState({ expanded: true, top: 0, left: 0 });
    });
  }

  render () {
    const { expanded, top, left } = this.state;
    const { children, className } = this.props;

    const classes = [ styles.overlay, className ];

    if (expanded) {
      classes.push(styles.expanded);
    }

    return (
      <Portal isOpened onClose={ this.handleClose }>
        <div
          className={ classes.join(' ') }
          style={ { top, left } }
          onKeyDown={ this.handleKeyDown }
        >
          <ParityBackground className={ styles.parityBackground } />

          { this.renderCloseIcon() }
          { children }
        </div>
      </Portal>
    );
  }

  renderCloseIcon () {
    const { expanded } = this.state;

    if (!expanded) {
      return null;
    }

    return (
      <div className={ styles.closeIcon } onClick={ this.handleClose }>
        <CloseIcon />
      </div>
    );
  }

  handleClose = () => {
    this.setPosition({ expanded: false }, this.props.onClose, 250);
  }

  handleKeyDown = (event) => {
    const codeName = keycode(event);

    switch (codeName) {
      case 'esc':
        event.preventDefault();
        return this.handleClose();

      default:
        event.persist();
        return this.props.onKeyDown(event);
    }
  }

  handleDOMAction = (ref, method) => {
    const refItem = typeof ref === 'string' ? this.refs[ref] : ref;
    const element = ReactDOM.findDOMNode(refItem);

    if (!element || typeof element[method] !== 'function') {
      console.warn('could not find', ref, 'or method', method);
      return;
    }

    return element[method]();
  }

  setPosition = (nextProps, callback = noop, callbackTimeout = 0) => {
    const { target } = this.props;
    const { top = 0, left = 0 } = this.handleDOMAction(target, 'getBoundingClientRect') || {};

    this.setState({ top, left, ...nextProps }, () => {
      window.setTimeout(callback, callbackTimeout);
    });
  }
}
