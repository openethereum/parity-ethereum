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

export default class Contract extends Component {
  static propTypes = {
    contract: PropTypes.object.isRequired,
    disabled: PropTypes.bool
  }

  render () {
    const { contract, disabled } = this.props;
    const location = contract.id === 'registry'
      ? 'chain'
      : 'registry';

    return (
      <ListItem
        disabled={ disabled }
        status={ contract.status }
      >
        <Header
          isBusy={ contract.isDeploying }
          isOk={ !!contract.instance && contract.isOnChain }
        >
          { contract.id } was {
            contract.address
              ? 'deployed'
              : 'not found'
          }
        </Header>
        <Row
          disabled={ !contract.instance }
          isBusy={ contract.isDeploying }
          isOk={ !!contract.address }
        >
          {
            contract.address
              ? contract.address
              : 'no address'
          }
        </Row>
        <Row
          disabled={ !contract.instance }
          isBusy={ contract.isDeploying }
          isOk={ !!contract.isOnChain }
        >
          {
            contract.isOnChain
              ? `registered on ${location}`
              : `not registered on ${location}`
          }
        </Row>
      </ListItem>
    );
  }
}
