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

import { Balance, Container, ContainerTitle, IdentityIcon, IdentityName, Tags } from '../../../ui';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    account: PropTypes.object.isRequired,
    balance: PropTypes.object.isRequired,
    link: PropTypes.string,
    children: PropTypes.node,
    handleAddSearchToken: PropTypes.func
  }

  state = {
    name: 'Unnamed'
  }

  render () {
    const { account, balance, children, link, handleAddSearchToken } = this.props;
    const { tags } = account.meta;

    if (!account) {
      return null;
    }

    const { address } = account;
    const viewLink = `/${link || 'account'}/${address}`;

    return (
      <Container>
        <Tags tags={ tags } handleAddSearchToken={ handleAddSearchToken } />
        <IdentityIcon
          address={ address } />
        <ContainerTitle
          title={ <Link to={ viewLink }>{ <IdentityName address={ address } unknown /> }</Link> }
          byline={ address } />
        <Balance
          balance={ balance } />
        { children }
      </Container>
    );
  }
}
