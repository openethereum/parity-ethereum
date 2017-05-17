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

// TODO: semantic-ui-react exposes Portal component, use that
import React, { Component, PropTypes } from 'react';
import ReactDOM from 'react-dom';
import ReactPortal from 'react-portal';
import keycode from 'keycode';
import { noop } from 'lodash';

import { nodeOrStringProptype } from '@parity/shared/util/proptypes';

import { CloseIcon } from '~/ui/Icons';
import StackEventListener from '~/ui/StackEventListener';
import Title from '~/ui/Title';

import styles from './portal.css';

export default class Portal extends Component {
  static propTypes = {
    open: PropTypes.bool.isRequired,
    activeStep: PropTypes.number,
    busy: PropTypes.bool,
    busySteps: PropTypes.array,
    buttons: PropTypes.oneOfType([
      PropTypes.array,
      PropTypes.node,
      PropTypes.object
    ]),
    children: PropTypes.node,
    className: PropTypes.string,
    hideClose: PropTypes.bool,
    isChildModal: PropTypes.bool,
    isSmallModal: PropTypes.bool,
    onClick: PropTypes.func,
    onClose: PropTypes.func,
    onKeyDown: PropTypes.func,
    steps: PropTypes.array,
    title: nodeOrStringProptype()
  };

  static defaultProps = {
    onClose: noop
  };

  componentDidMount () {
    this.setBodyOverflow(this.props.open);
  }

  componentWillReceiveProps (nextProps) {
    if (nextProps.open !== this.props.open) {
      this.setBodyOverflow(nextProps.open);
    }
  }

  componentWillUnmount () {
    this.setBodyOverflow(false);
  }

  render () {
    const { activeStep, busy, busySteps, buttons, children, className, hideClose, isChildModal, isSmallModal, open, steps, title } = this.props;

    if (!open) {
      return null;
    }

    return (
      <ReactPortal isOpened>
        <div
          className={ styles.backOverlay }
          onClick={ this.handleClose }
        >
          <div
            className={
              [
                styles.overlay,
                isChildModal
                  ? styles.popover
                  : styles.modal,
                isSmallModal
                  ? styles.small
                  : null,
                className
              ].join(' ')
            }
            onClick={ this.handleContainerClick }
            onKeyDown={ this.handleKeyDown }
          >
            <StackEventListener onKeyUp={ this.handleKeyUp } />
            {
              !hideClose && (
                <div
                  className={ styles.closeIcon }
                  onClick={ this.handleClose }
                >
                  <CloseIcon />
                </div>
              )
            }
            <Title
              activeStep={ activeStep }
              busy={ busy }
              busySteps={ busySteps }
              className={ styles.titleRow }
              steps={ steps }
              title={ title }
            />
            <div className={ styles.childContainer }>
              { children }
            </div>
            {
              buttons && (
                <div className={ styles.buttonRow }>
                  { buttons }
                </div>
              )
            }
          </div>
        </div>
      </ReactPortal>
    );
  }

  stopEvent = (event) => {
    event.stopPropagation();
  }

  handleContainerClick = (event) => {
    const { onClick } = this.props;

    if (!onClick) {
      return this.stopEvent(event);
    }

    return onClick(event);
  }

  handleClose = () => {
    const { hideClose, onClose } = this.props;

    if (!hideClose) {
      onClose();
    }

    this.stopEvent(event);
  }

  handleKeyDown = (event) => {
    const { onKeyDown } = this.props;

    event.persist();

    return onKeyDown
      ? onKeyDown(event)
      : false;
  }

  handleKeyUp = (event) => {
    const codeName = keycode(event);

    switch (codeName) {
      case 'esc':
        event.preventDefault();
        return this.props.onClose();
    }
  }

  handleDOMAction = (ref, method) => {
    const element = ReactDOM.findDOMNode(
      typeof ref === 'string'
        ? this.refs[ref]
        : ref
    );

    if (!element || typeof element[method] !== 'function') {
      console.warn('could not find', ref, 'or method', method);
      return;
    }

    return element[method]();
  }

  setBodyOverflow (open) {
    if (!this.props.isChildModal) {
      document.body.style.overflow = open
        ? 'hidden'
        : null;
    }
  }
}
