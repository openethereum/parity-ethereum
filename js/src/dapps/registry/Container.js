import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import Application from './Application';
import * as actions from './actions';

class Container extends Component {
  static propTypes = {
    actions: PropTypes.object,
    contract: PropTypes.object,
    owner: PropTypes.string,
    fee: PropTypes.object
  };
  componentDidMount () {
    this.props.actions.fetchContract();
  }

  render () {
    const { actions, contract, owner, fee } = this.props
    return (<Application
      actions={ actions }
      contract={ contract }
      owner={ owner }
      fee={ fee }
    />);
  }
}

export default connect(
  // redux -> react connection
  (state) => state,
  // react -> redux connection
  (dispatch) => ({ actions: bindActionCreators(actions, dispatch) })
)(Container);
