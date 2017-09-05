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

import Application from './Application';

import { loadContract } from './Status/actions';
import { loadAccounts } from './Accounts/actions';

class Container extends Component {
  static propTypes = {
    isLoading: PropTypes.bool.isRequired,
    contract: PropTypes.object.isRequired,
    onLoad: PropTypes.func.isRequired
  };

  componentDidMount () {
    this.props.onLoad();
  }

  render () {
    const { isLoading, contract } = this.props;

    return (
      <Application
        isLoading={ isLoading }
        contract={ contract }
      />
    );
  }
}

const mapStateToProps = (state) => {
  const { isLoading, contract } = state.status;

  return {
    isLoading,
    contract
  };
};

const mapDispatchToProps = (dispatch) => {
  return {
    onLoad: () => {
      dispatch(loadContract());
      dispatch(loadAccounts());
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Container);
