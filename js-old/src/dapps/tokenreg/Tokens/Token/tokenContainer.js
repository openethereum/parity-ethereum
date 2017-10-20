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

import Token from './token';

import { queryTokenMeta, unregisterToken, addTokenMeta } from '../actions';

class TokenContainer extends Component {
  static propTypes = {
    handleMetaLookup: PropTypes.func.isRequired,
    handleUnregister: PropTypes.func.isRequired,
    handleAddMeta: PropTypes.func.isRequired,

    tla: PropTypes.string.isRequired
  };

  render () {
    return (
      <Token
        { ...this.props }
      />
    );
  }
}

const mapStateToProps = (_, initProps) => {
  const { tla } = initProps;

  return (state) => {
    const { isOwner } = state.status.contract;
    const { tokens } = state.tokens;
    const token = tokens.find((t) => t.tla === tla);

    return { ...token, isContractOwner: isOwner };
  };
};

const mapDispatchToProps = (dispatch) => {
  return {
    handleMetaLookup: (index, query) => {
      dispatch(queryTokenMeta(index, query));
    },

    handleUnregister: (index) => {
      dispatch(unregisterToken(index));
    },

    handleAddMeta: (index, key, value, validationType) => {
      dispatch(addTokenMeta(index, key, value, validationType));
    }
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TokenContainer);
