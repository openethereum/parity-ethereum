import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { Snackbar } from 'material-ui';

import { closeErrors } from './actions';

class Errors extends Component {
  static propTypes = {
    message: PropTypes.string,
    visible: PropTypes.bool,
    onCloseErrors: PropTypes.func
  };

  render () {
    const { message, visible } = this.props;

    if (!message || !visible) {
      return null;
    }

    return (
      <Snackbar
        open
        message={ message }
        autoHideDuration={ 5000 }
        onRequestClose={ this.props.onCloseErrors } />
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
