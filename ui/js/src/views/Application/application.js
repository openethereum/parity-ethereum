// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import Connection from '../Connection';
import ParityBar from '../ParityBar';

import Container from './Container';
import DappContainer from './DappContainer';
import FrameError from './FrameError';
import Status from './Status';
import TabBar from './TabBar';

import styles from './application.css';

const inFrame = window.parent !== window && window.parent.frames.length !== 0;
const showFirstRun = window.localStorage.getItem('showFirstRun') === '1';

class Application extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    background: PropTypes.string
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
    const [root] = (window.location.hash || '').replace('#/', '').split('/');
    const isDapp = root === 'app';

    if (inFrame) {
      return (
        <FrameError />
      );
    }

    return (
      <div className={ styles.outer }>
        { isDapp ? this.renderDapp() : this.renderApp() }
        <Connection />
        <ParityBar dapp={ isDapp } />
      </div>
    );
  }

  renderApp () {
    const { children, pending, netChain, isTest } = this.props;
    const { showFirstRun } = this.state;

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

  renderDapp () {
    const { children } = this.props;

    return (
      <DappContainer>
        { children }
      </DappContainer>
    );
  }

  checkAccounts () {
    const { api } = this.context;

    api.personal
      .listAccounts()
      .then((accounts) => {
        this.setState({
          showFirstRun: showFirstRun || accounts.length === 0
        });
      })
      .catch((error) => {
        console.error('checkAccounts', error);
      });
  }

  onCloseFirstRun = () => {
    window.localStorage.setItem('showFirstRun', '0');
    this.setState({
      showFirstRun: false
    });
  }
}

function mapStateToProps (state) {
  const { netChain, isTest } = state.nodeStatus;
  const { hasAccounts } = state.personal;
  const { pending } = state.signer;

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
