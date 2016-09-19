import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';

import AccountSelector from './account-selector';

import { setSelectedAccount } from '../actions';

class AccountSelectorContainer extends Component {
  static propTypes = {
    accounts: PropTypes.object
  };

  render () {
    return (<AccountSelector
      { ...this.props }
    />);
  }
}

const mapStateToProps = (state) => {
  const { accounts } = state;
  return { ...accounts };
};

const mapDispatchToProps = (dispatch) => {
  return {
    handleSetSelected: (address) => {
      dispatch(setSelectedAccount(address));
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(AccountSelectorContainer);
