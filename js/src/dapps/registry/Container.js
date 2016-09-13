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
    const { actions, accounts, account, contract, owner, fee, lookup, events, register } = this.props;
    return (<Application
      actions={ actions }
      accounts={ accounts }
      account={ account }
      contract={ contract }
      owner={ owner }
      fee={ fee }
      lookup={ lookup }
      events={ events }
      register={ register }
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
