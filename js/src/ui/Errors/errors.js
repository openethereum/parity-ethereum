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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { Snackbar } from 'material-ui';

import { closeErrors } from './actions';

import styles from './errors.css';

class Errors extends Component {
  static propTypes = {
    message: PropTypes.string,
    visible: PropTypes.bool,
    onCloseErrors: PropTypes.func
  };

  render () {
    const { message, visible, onCloseErrors } = this.props;

    if (!message || !visible) {
      return null;
    }

    return (
      <Snackbar
        open
        className={ styles.container }
        message={ message }
        autoHideDuration={ 5000 }
        onRequestClose={ onCloseErrors } />
    );
  }
}

function mapStateToProps (state) {
  const { message, visible } = state.errors;

  return {
    message,
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
