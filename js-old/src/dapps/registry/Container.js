// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

import { nullableProptype } from '~/util/proptypes';

import Application from './Application';
import * as actions from './actions';

class Container extends Component {
  static propTypes = {
    actions: PropTypes.object.isRequired,
    accounts: PropTypes.object.isRequired,
    contacts: PropTypes.object.isRequired,
    contract: nullableProptype(PropTypes.object.isRequired),
    owner: nullableProptype(PropTypes.string.isRequired),
    fee: nullableProptype(PropTypes.object.isRequired),
    lookup: PropTypes.object.isRequired,
    events: PropTypes.object.isRequired
  };

  componentDidMount () {
    Promise.all([
      this.props.actions.fetchIsTestnet(),
      this.props.actions.addresses.fetch(),
      this.props.actions.fetchContract()
    ]).then(() => {
      this.props.actions.events.subscribe('Reserved');
    });
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
    bound.names = bindActionCreators(actions.names, dispatch);
    bound.records = bindActionCreators(actions.records, dispatch);
    return { actions: bound };
  }
)(Container);
