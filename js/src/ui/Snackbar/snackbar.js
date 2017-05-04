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
import { FormattedMessage } from 'react-intl';

import styles from './snackbar.css';

export default class Snackbar extends Component {
  state = {
    snackStyle: {
      transform: 'translateX(-50%) translateY(40px)'
    }
  };

  static propTypes = {
    action: PropTypes.any,
    open: PropTypes.bool,
    message: PropTypes.string,
    autoHideDuration: PropTypes.number,
    bodyStyle: PropTypes.object,
    onRequestClose: PropTypes.Func
  };

  defaultProps = {
    autoHideDuration: 3500
  };

  componentWillUpdate (nextProps) {
    const self = this;

    if (this.openStatus) {
      return;
    }

    if (nextProps.open === true) {
      this.openStatus = true;

      self.autoShow();

      setTimeout(() => {
        self.autoHide();
      }, nextProps.autoHideDuration);
    }
  }

  autoShow () {
    this.setState({
      snackStyle: {
        transform: 'translateX(-50%) translateY(0px)'
      }
    });
  }

  autoHide () {
    this.props.onRequestClose();
    this.openStatus = false;
    this.setState({
      snackStyle: {
        transform: 'translateX(-50%) translateY(40px)'
      }
    });
  }

  render () {
    const { bodyStyle, message } = this.props;
    const { snackStyle } = this.state;
    let { action } = this.props;

    if (action === null || action === 'undefined') {
      action = (
        <FormattedMessage
          id='ui.snackbar.close'
          defaultMessage='close'
        />
      );
    }

    return (
      <div className={ styles.snacks } style={ snackStyle }>
        <div style={ bodyStyle }>
          <span>{ message }</span>
          <span id={ styles.action } onClick={ this.autoHide }>{ action }</span>
        </div>
      </div>
    );
  }
}
