import React, { Component } from 'react';
import { connect } from 'react-redux';

import Actions from './component';

import { registerToken, registerReset } from './actions';

// import { loadTokens, queryTokenMeta } from './actions';

class TokensContainer extends Component {

  render () {
    return (<Actions
      { ...this.props }
    />);
  }
}

const mapStateToProps = (state) => {
  const { register } = state.actions;

  return { register };
};

const mapDispatchToProps = (dispatch) => {
  return {
    handleRegisterToken: (tokenData) => {
      dispatch(registerToken(tokenData));
    },
    handleRegisterClose: () => {
      dispatch(registerReset());
    }
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TokensContainer);
