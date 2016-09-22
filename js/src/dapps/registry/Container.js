import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import Application from './Application';
import * as actions from './actions';

const nullable = (type) => React.PropTypes.oneOfType([ React.PropTypes.oneOf([ null ]), type ]);

class Container extends Component {
  static propTypes = {
    actions: PropTypes.object.isRequired,
    accounts: PropTypes.object.isRequired,
    contacts: PropTypes.object.isRequired,
    contract: nullable(PropTypes.object.isRequired),
    owner: nullable(PropTypes.string.isRequired),
    fee: nullable(PropTypes.object.isRequired),
    lookup: PropTypes.object.isRequired,
    events: PropTypes.object.isRequired
  };

  componentDidMount () {
    this.props.actions.addresses.fetch();
    this.props.actions.fetchContract();
  }

  render () {
    return (<Application { ...this.props } />);
  }
}

export default connect(
  // redux -> react connection
  (state) => state,
  // react -> redux connection
  (dispatch) => {
    const bound = bindActionCreators(actions, dispatch);
    bound.addresses = bindActionCreators(actions.addresses, dispatch);
    bound.accounts = bindActionCreators(actions.accounts, dispatch);
    bound.lookup = bindActionCreators(actions.lookup, dispatch);
    bound.events = bindActionCreators(actions.events, dispatch);
    bound.register = bindActionCreators(actions.register, dispatch);
    return { actions: bound };
  }
)(Container);
