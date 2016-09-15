import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';

import Application from './Application';

import { loadContract } from './Status/actions';
import { loadAccounts } from './Accounts/actions';

class Container extends Component {
  static propTypes = {
    isLoading: PropTypes.bool,
    contract: PropTypes.object
  };

  componentDidMount() {
    this.props.onLoad();
  }

  render() {
    const { isLoading, contract } = this.props;

    return (<Application
      isLoading={ isLoading }
      contract={ contract }
    />);
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
