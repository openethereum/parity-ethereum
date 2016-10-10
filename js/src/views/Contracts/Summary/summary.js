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
import { Link } from 'react-router';

import { Container, ContainerTitle, IdentityIcon, IdentityName } from '../../../ui';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object.isRequired
  }

  static propTypes = {
    contract: PropTypes.object.isRequired,
    children: PropTypes.node
  }

  render () {
    const contract = this.props.contract;

    if (!contract) {
      return null;
    }

    const viewLink = `/app/${contract.address}`;

    return (
      <Container>
        <IdentityIcon
          address={ contract.address } />
        <ContainerTitle
          title={ <Link to={ viewLink }>{ <IdentityName address={ contract.address } unknown /> }</Link> }
          byline={ contract.address } />
        { this.props.children }
      </Container>
    );
  }
}
