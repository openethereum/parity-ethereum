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

import { observer } from 'mobx-react';
import React, { Component } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import PropTypes from 'prop-types';

import HardwareStore from '@parity/shared/lib/mobx/hardwareStore';
import UpgradeStore from '@parity/shared/lib/mobx/upgradeParity';
import Errors from '@parity/ui/lib/Errors';

import Connection from '../Connection';
import DappRequests from '../DappRequests';
import Extension from '../Extension';
import FirstRun from '../FirstRun';
import ParityBar from '../ParityBar';
import PinMatrix from '../PinMatrix';
import Requests from '../Requests';
import Snackbar from '../Snackbar';
import Status from '../Status';
import UpgradeParity from '../UpgradeParity';

import { appLogoDark as parityLogo } from '../config';
import styles from './application.css';

const inFrame = window.parent !== window && window.parent.frames.length !== 0;

@observer
class Application extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    background: PropTypes.string
  }

  static propTypes = {
    blockNumber: PropTypes.object,
    children: PropTypes.node,
    pending: PropTypes.array
  }

  hwstore = HardwareStore.get(this.context.api);
  upgradeStore = UpgradeStore.get(this.context.api);

  render () {
    const { blockNumber } = this.props;
    const [root] = (window.location.hash || '').replace('#/', '').split('/');
    const isMinimized = root !== '';
    const { pinMatrixRequest } = this.hwstore;

    if (inFrame) {
      return (
        <div className={ styles.error }>
          <FormattedMessage
            id='application.frame.error'
            defaultMessage='ERROR: This application cannot and should not be loaded in an embedded iFrame'
          />
        </div>
      );
    }

    return (
      <div className={ styles.application }>
        {
          blockNumber
            ? <Status upgradeStore={ this.upgradeStore } />
            : null
        }
        {
          isMinimized
            ? this.renderMinimized()
            : this.renderApp()
        }
        <Connection />
        <DappRequests />
        {
          (pinMatrixRequest.length > 0)
            ? (
              <PinMatrix
                device={ pinMatrixRequest[0] }
                store={ this.hwstore }
              />
            )
            : null
        }
        <Requests />
        <ParityBar
          alwaysHidden
          dapp={ isMinimized }
        />
      </div>
    );
  }

  renderApp () {
    const { children } = this.props;

    return (
      <div className={ styles.container }>
        <Extension />
        <FirstRun />
        <Snackbar />
        <UpgradeParity upgradeStore={ this.upgradeStore } />
        <Errors />
        <div className={ styles.content }>
          { children }
        </div>
      </div>
    );
  }

  renderMinimized () {
    const { children } = this.props;

    return (
      <div className={ styles.container }>
        <div className={ styles.logo }>
          <img src={ parityLogo } />
        </div>
        <Errors />
        { children }
      </div>
    );
  }
}

function mapStateToProps (state) {
  const { blockNumber } = state.nodeStatus;

  return {
    blockNumber
  };
}

export default connect(
  mapStateToProps,
  null
)(Application);
