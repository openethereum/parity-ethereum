import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import Application from './Application';
import * as actions from './actions';

class Container extends Component {
  static propTypes = {
    actions: PropTypes.object,
    accounts: PropTypes.object,
    account: PropTypes.object,
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
    const props = this.props;
    return (<Application
      actions={ props.actions }
      accounts={ props.accounts }
      contract={ props.contract }
      owner={ props.owner }
      fee={ props.fee }
      lookup={ props.lookup }
      events={ props.events }
      register={ props.register }
    />);
  }
}

export default connect(
  // redux -> react connection
  (state) => state,
  // react -> redux connection
  (dispatch) => {
    const bound = bindActionCreators(actions, dispatch);
    bound.accounts = bindActionCreators(actions.accounts, dispatch);
    bound.lookup = bindActionCreators(actions.lookup, dispatch);
    bound.events = bindActionCreators(actions.events, dispatch);
    bound.register = bindActionCreators(actions.register, dispatch);
    return { actions: bound };
  }
)(Container);
