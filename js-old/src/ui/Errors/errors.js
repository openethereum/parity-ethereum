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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { Snackbar } from 'material-ui';

import { closeErrors } from './actions';

import styles from './errors.css';

const ERROR_REGEX = /-(\d+): (.+)$/;
const DURATION_OPEN = 60000;
const STYLE_BODY = {
  height: 'auto',
  whiteSpace: 'pre-line'
};
const STYLE_CONTENT = {
  alignItems: 'center',
  display: 'flex',
  flexDirection: 'row',
  justifyContent: 'space-between',
  lineHeight: '1.5em',
  padding: '0.75em 0'
};

class Errors extends Component {
  static propTypes = {
    message: PropTypes.string,
    error: PropTypes.object,
    visible: PropTypes.bool,
    onCloseErrors: PropTypes.func
  };

  render () {
    const { message, visible, onCloseErrors } = this.props;

    if (!message || !visible) {
      return null;
    }

    const text = this.getErrorMessage();

    return (
      <Snackbar
        className={ styles.container }
        open
        action={
          <FormattedMessage
            id='ui.errors.close'
            defaultMessage='close'
          />
        }
        autoHideDuration={ DURATION_OPEN }
        message={ text }
        onActionTouchTap={ onCloseErrors }
        onRequestClose={ this.onRequestClose }
        bodyStyle={ STYLE_BODY }
        contentStyle={ STYLE_CONTENT }
      />
    );
  }

  getErrorMessage = () => {
    const { message, error } = this.props;

    if (!error.text && !ERROR_REGEX.test(message)) {
      return message;
    }

    const matches = ERROR_REGEX.exec(message);

    const code = error.code || parseInt(matches[1]) * -1;
    const text = error.text || matches[2];

    return `[${code}] ${text}`;
  }

  onRequestClose = (reason) => {
    if (reason === 'timeout') {
      this.props.onCloseErrors();
    }
  }
}

function mapStateToProps (state) {
  const { message, error, visible } = state.errors;

  return {
    message,
    error,
    visible
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    onCloseErrors: closeErrors
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Errors);
