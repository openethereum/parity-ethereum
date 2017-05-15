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
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import store from 'store';

import { Button, Checkbox } from '@parity/ui';

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
    isSyncing: PropTypes.bool
  };

  state = {
    dontShowAgain: false,
    show: true
  };

  render () {
    const { isSyncing } = this.props;
    const { dontShowAgain, show } = this.state;

    if (!isSyncing || isSyncing === null || !show) {
      return null;
    }

    return (
      <div>
        <div className={ styles.overlay } />
        <div className={ styles.modal }>
          <div className={ styles.body }>
            <FormattedMessage
              id='syncWarning.message.line1'
              defaultMessage={ `
                Your Parity node is still syncing to the chain.
              ` }
            />
            <FormattedMessage
              id='syncWarning.message.line2'
              defaultMessage={ `
                Some of the shown information might be out-of-date.
              ` }
            />

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
  const { syncing } = state.nodeStatus;
  // syncing could be an Object, false, or null
  const isSyncing = syncing
    ? true
    : syncing;

  return {
    isSyncing
  };
}

export default connect(
  mapStateToProps,
  null
)(SyncWarning);
