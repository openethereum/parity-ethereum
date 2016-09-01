import React, { Component } from 'react';
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';

import { updateToken } from '../../actions/app';

import UnAuthorized from '../../components/UnAuthorized';

class UnAuthorizedPage extends Component {
  render () {
    return (
      <UnAuthorized { ...this.props } />
    );
  }
}

function mapStateToProps (state) {
  return state;
}

function mapDispatchToProps (dispatch) {
  return {
    actions: bindActionCreators({ updateToken }, dispatch)
  };
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(UnAuthorizedPage);
