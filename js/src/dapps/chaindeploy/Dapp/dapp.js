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

import styles from './dapp.css';

export default class Dapp extends Component {
  static propTypes = {
    dapp: PropTypes.object.isRequired,
    disabled: PropTypes.bool
  }

  render () {
    const { dapp, disabled } = this.props;

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
          <div className={ styles.icon }>
            {
              dapp.isOnChain
                ? '\u2714'
                : (
                  dapp.isDeploying
                    ? '\u29d6'
                    : '\u2716'
                  )
            }
          </div>
          <div className={ styles.title }>
            { dapp.name } was {
              dapp.isOnChain
                ? 'found in dappreg'
                : 'not found'
            }
          </div>
        </div>
        <div
          className={
            [
              styles.details,
              dapp.isOnChain
                ? ''
                : styles.muted
            ].join(' ') }
        >
          <div className={ styles.icon }>
            {
              dapp.imageHash
                ? '\u2714'
                : (
                  dapp.isDeploying
                    ? '\u29d6'
                    : '\u2716'
                  )
            }
          </div>
          <div className={ styles.title }>
            {
              dapp.imageHash
                ? `registered imageHash ${dapp.imageHash}`
                : 'has not registered an imageHash'
            }
          </div>
        </div>
        <div
          className={
            [
              styles.details,
              dapp.imageHash
                ? ''
                : styles.muted
            ].join(' ') }
        >
          <div className={ styles.icon }>
            {
              dapp.imageUrl
                ? '\u2714'
                : (
                  dapp.isDeploying
                    ? '\u29d6'
                    : '\u2716'
                  )
            }
          </div>
          <div className={ styles.title }>
            {
              dapp.imageUrl
                ? `resolving imageUrl ${dapp.imageUrl}`
                : 'does not resolve imageUrl'
            }
          </div>
        </div>
      </div>
    );
  }
}
