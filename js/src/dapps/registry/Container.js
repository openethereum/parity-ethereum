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
    fee: PropTypes.object,
    lookup: PropTypes.object,
    events: PropTypes.array
  };
  componentDidMount () {
    this.props.actions.fetchContract();
  }

  render () {
    const { actions, contract, owner, fee, lookup, events } = this.props;
    return (<Application
      actions={ actions }
      contract={ contract }
      owner={ owner }
      fee={ fee }
      lookup={ lookup }
      events={ events }
    />);
  }
}

export default connect(
  // redux -> react connection
  (state) => state,
  // react -> redux connection
  (dispatch) => {
    const bound = bindActionCreators(actions, dispatch);
    bound.lookup = bindActionCreators(actions.lookup, dispatch);
    bound.events = bindActionCreators(actions.events, dispatch);
    return { actions: bound };
  }
)(Container);
