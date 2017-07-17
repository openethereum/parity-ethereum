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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import styles from './snackbar.css';

export default class Snackbar extends Component {
  state = {
    opened: false
  };

  static propTypes = {
    action: PropTypes.any,
    open: PropTypes.bool,
    message: PropTypes.string,
    autoHideDuration: PropTypes.number,
    bodyStyle: PropTypes.object,
    onRequestClose: PropTypes.func
  };

  defaultProps = {
    autoHideDuration: 3500
  };

  componentWillUpdate (nextProps) {
    if (this.state.opened) {
      return;
    }

    if (nextProps.open === true) {
      this.show();

      setTimeout(this.hide, nextProps.autoHideDuration);
    }
  }

  render () {
    const { bodyStyle, message } = this.props;
    const { opened } = this.state;
    let { action } = this.props;

    if (!opened) {
      return false;
    }

    if (action === null || action === 'undefined') {
      action = (
        <FormattedMessage
          id='ui.snackbar.close'
          defaultMessage='close'
        />
      );
    }

    return (
      <div className={ styles.snacks }>
        <div style={ bodyStyle }>
          <span>{ message }</span>
          <span id={ styles.action } onClick={ this.autoHide }>{ action }</span>
        </div>
      </div>
    );
  }

  show = () => {
    this.setState({
      opened: true
    });
  }

  hide = () => {
    this.props.onRequestClose();
    this.setState({
      opened: false
    });
  }
}
