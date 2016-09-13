import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import Container from './Container';
import DappContainer from './DappContainer';
import FrameError from './FrameError';
import Status from './Status';
import TabBar from './TabBar';

const inFrame = window.parent !== window && window.parent.frames.length !== 0;

class Application extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    children: PropTypes.node,
    netChain: PropTypes.string,
    isTest: PropTypes.bool,
    pending: PropTypes.array
  }

  state = {
    showFirstRun: false
  }

  componentWillMount () {
    this.checkAccounts();
  }

  render () {
    const { children, pending, netChain, isTest } = this.props;
    const { showFirstRun } = this.state;
    const [root] = (window.location.hash || '').replace('#/', '').split('/');

    if (inFrame) {
      return (
        <FrameError />
      );
    } else if (root === 'app') {
      return (
        <DappContainer>
          { children }
        </DappContainer>
      );
    }

    return (
      <Container
        showFirstRun={ showFirstRun }
        onCloseFirstRun={ this.onCloseFirstRun }>
        <TabBar
          netChain={ netChain }
          isTest={ isTest }
          pending={ pending } />
        { children }
        <Status />
      </Container>
    );
  }

  checkAccounts () {
    const { api } = this.context;

    api.personal
      .listAccounts()
      .then((accounts) => {
        this.setState({
          showFirst: accounts.length === 0
        });
      })
      .catch((error) => {
        console.error('checkAccounts', error);
      });
  }

  onCloseFirstRun = () => {
    this.setState({
      showFirstRun: false
    });
  }
}

function mapStateToProps (state) {
  const { netChain, isTest } = state.nodeStatus;
  const { hasAccounts } = state.personal;
  const { pending } = state.signerRequests;

  return {
    hasAccounts,
    netChain,
    isTest,
    pending
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Application);
