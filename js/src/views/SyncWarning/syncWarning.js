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

import { Checkbox } from 'material-ui';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import store from 'store';

import { Button, StatusIndicator } from '~/ui';

import styles from './syncWarning.css';

const LS_DONT_SHOW_AGAIN = '_parity::syncWarning::dontShowAgain';

export const showSyncWarning = () => {
  const dontShowAgain = store.get(LS_DONT_SHOW_AGAIN);

  if (dontShowAgain === undefined || dontShowAgain === null) {
    return true;
  }

  return !dontShowAgain;
};

class SyncWarning extends Component {
  static propTypes = {
    isOk: PropTypes.bool.isRequired,
    health: PropTypes.object.isRequired
  };

  state = {
    dontShowAgain: false,
    show: true
  };

  render () {
    const { isOk, health } = this.props;
    const { dontShowAgain, show } = this.state;

    if (isOk || !show) {
      return null;
    }

    return (
      <div>
        <div className={ styles.overlay } />
        <div className={ styles.modal }>
          <div className={ styles.body }>
            <div className={ styles.status }>
              <StatusIndicator
                type='signal'
                id='healthWarning.indicator'
                status={ health.overall.status }
              />
            </div>

            {
              health.overall.message.map(message => (
                <p key={ message }>{ message }</p>
              ))
            }

            <div className={ styles.button }>
              <Checkbox
                label={
                  <FormattedMessage
                    id='syncWarning.dontShowAgain.label'
                    defaultMessage='Do not show this warning again'
                  />
                }
                checked={ dontShowAgain }
                onCheck={ this.handleCheck }
              />
              <Button
                label={
                  <FormattedMessage
                    id='syncWarning.understandBtn.label'
                    defaultMessage='I understand'
                  />
                }
                onClick={ this.handleAgreeClick }
              />
            </div>
          </div>
        </div>
      </div>
    );
  }

  handleCheck = () => {
    this.setState({ dontShowAgain: !this.state.dontShowAgain });
  }

  handleAgreeClick = () => {
    if (this.state.dontShowAgain) {
      store.set(LS_DONT_SHOW_AGAIN, true);
    }

    this.setState({ show: false });
  }
}

function mapStateToProps (state) {
  const { health } = state.nodeStatus;
  const isNotAvailableYet = health.overall.isNotReady;
  const isOk = isNotAvailableYet || health.overall.status === 'ok';

  return {
    isOk,
    health
  };
}

export default connect(
  mapStateToProps,
  null
)(SyncWarning);
