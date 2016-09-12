import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import Container from './Container';
import DappContainer from './DappContainer';
import FrameError from './FrameError';
import Status, { updateNodeStatus } from './Status';
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
    onUpdateNodeStatus: PropTypes.func,
    pending: PropTypes.array
  }

  state = {
    showFirstRun: false
  }

  componentWillMount () {
    this.pollStatus();
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

  pollStatus () {
    const { api } = this.context;
    const { onUpdateNodeStatus } = this.props;
    const nextTimeout = () => setTimeout(() => this.pollStatus(), 1000);

    Promise
      .all([
        api.eth.blockNumber(),
        api.web3.clientVersion(),
        api.ethcore.netChain(),
        api.ethcore.netPeers(),
        api.eth.syncing()
      ])
      .then(([blockNumber, clientVersion, netChain, netPeers, syncing]) => {
        const isTest = netChain === 'morden' || netChain === 'testnet';

        onUpdateNodeStatus({
          blockNumber,
          clientVersion,
          netChain,
          netPeers,
          isTest,
          syncing
        });

        nextTimeout();
      })
      .catch((error) => {
        console.error('pollStatus', error);

        nextTimeout();
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
  return bindActionCreators({
    onUpdateNodeStatus: updateNodeStatus
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Application);
