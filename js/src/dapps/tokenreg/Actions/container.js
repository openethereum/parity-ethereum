import React, { Component } from 'react';
import { connect } from 'react-redux';

import Actions from './component';

import { registerToken, registerReset, queryToken, queryReset, queryTokenMeta } from './actions';

class TokensContainer extends Component {

  render () {
    return (<Actions
      { ...this.props }
    />);
  }
}

const mapStateToProps = (state) => {
  const { register, query } = state.actions;

  return { register, query };
};

const mapDispatchToProps = (dispatch) => {
  return {
    handleRegisterToken: (tokenData) => {
      dispatch(registerToken(tokenData));
    },
    handleRegisterClose: () => {
      dispatch(registerReset());
    },
    handleQueryToken: (key, query) => {
      dispatch(queryToken(key, query));
    },
    handleQueryClose: () => {
      dispatch(queryReset());
    },
    handleQueryMetaLookup: (id, query) => {
      dispatch(queryTokenMeta(id, query));
    }
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TokensContainer);
