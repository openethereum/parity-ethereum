
import React, { Component, PropTypes } from 'react';
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';
import { extend } from 'lodash';

import * as debugActions from '../../actions/debug';
import { updateLogging } from '../../actions/logger';
import Debug from '../../components/Debug';

class DebugPage extends Component {

  render () {
    return <Debug { ...this.props } />;
  }

  static propTypes = {
    actions: PropTypes.object.isRequired,
    debug: PropTypes.object.isRequired
  }

}

function mapStateToProps (state) {
  return state;
}

function mapDispatchToProps (dispatch) {
  return {
    actions: bindActionCreators(extend({}, debugActions, { updateLogging }), dispatch)
  };
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(DebugPage);
