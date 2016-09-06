import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import actions from './actions';

class Container extends Component {

  componentDidMount () {
    this.props.fetchContract();
  }

  render () {
    return (<div>{ this.props.foo }</div>);
  }

}

Container.propTypes = {
  foo: PropTypes.string
};

export default connect(
  // redux -> react connection
  (state) => state,
  // react -> redux connection
  (dispatch) => bindActionCreators(actions, dispatch)
)(Container);
