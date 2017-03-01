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
import BusyIcon from 'material-ui/svg-icons/action/schedule';
import DoneIcon from 'material-ui/svg-icons/action/done';
import UnknownIcon from 'material-ui/svg-icons/action/help-outline';

import styles from './contract.css';

export default class Contract extends Component {
  static propTypes = {
    contract: PropTypes.object.isRequired,
    disabled: PropTypes.bool
  }

  render () {
    const { contract, disabled } = this.props;

    return (
      <div
        className={
          [
            styles.listItem,
            disabled
              ? styles.muted
              : ''
          ].join(' ')
        }
      >
        <div className={ styles.header }>
          {
            contract.address
              ? <DoneIcon className={ styles.icon } />
              : (
                contract.isDeploying
                  ? <BusyIcon className={ styles.icon } />
                  : <UnknownIcon className={ styles.icon } />
                )
          }
          <div className={ styles.title }>
            { contract.id } was {
              contract.address
                ? `deployed at ${contract.address}`
                : 'not found'
            }
          </div>
        </div>
        <div
          className={
            [
              styles.details,
              contract.address
                ? ''
                : styles.muted
            ].join(' ') }
        >
          {
            contract.isOnChain
              ? <DoneIcon className={ styles.icon } />
              : <UnknownIcon className={ styles.icon } />
          }
          <div className={ styles.title }>
            {
              contract.isOnChain
                ? 'registered on chain'
                : 'not registered on chain'
            }
          </div>
        </div>
      </div>
    );
  }
}
