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

import ListItem, { Header, Row } from '../ListItem';

export default class Dapp extends Component {
  static propTypes = {
    dapp: PropTypes.object.isRequired,
    disabled: PropTypes.bool
  }

  render () {
    const { dapp, disabled } = this.props;

    return (
      <ListItem
        disabled={ disabled }
        status={ dapp.status }
      >
        <Header
          isBusy={ dapp.isDeploying }
          isOk={
            dapp.isOnChain &&
            !!dapp.imageHash && !!dapp.imageMatch &&
            (!dapp.source.contentHash || !!dapp.contentMatch) &&
            (!dapp.source.manifestHash || !!dapp.manifestMatch)
          }
        >
          { dapp.name }
        </Header>
        <Row
          isBusy={ dapp.isDeploying }
          isOk={ dapp.isOnChain }
        >
          {
            dapp.isOnChain
              ? 'found in dappreg'
              : 'not found in dappreg'
          }
        </Row>
        { this.renderHash(dapp, 'image') }
        { this.renderHash(dapp, 'manifest') }
        { this.renderHash(dapp, 'content') }
      </ListItem>
    );
  }

  renderHash (dapp, type) {
    if (!dapp.source[`${type}Hash`]) {
      return null;
    }

    const isMatch = dapp[`${type}Match`];
    const hash = dapp[`${type}Hash`];

    return [
      <Row
        disabled={ !dapp.isOnChain }
        isBusy={ dapp.isDeploying }
        isOk={ !!hash }
        key={ `${type}Hash` }
      >
        {
          hash
            ? `${type}Hash ${hash}`
            : `has not registered an ${type}Hash`
        }
      </Row>,
      <Row
        disabled={ !dapp.isOnChain }
        isBusy={ dapp.isDeploying }
        isOk={ isMatch }
        key={ `${type}Match` }
      >
        {
          isMatch
            ? `has latest ${type}Hash`
            : `does not have latest ${type}Hash`
        }
      </Row>
    ];
  }
}
