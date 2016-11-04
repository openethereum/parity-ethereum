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
import { isEqual } from 'lodash';

import { Balance, Container, ContainerTitle, IdentityIcon, IdentityName, Tags, Input } from '../../../ui';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  };

  static propTypes = {
    account: PropTypes.object.isRequired,
    balance: PropTypes.object.isRequired,
    link: PropTypes.string,
    children: PropTypes.node,
    handleAddSearchToken: PropTypes.func
  };

  shouldComponentUpdate (nextProps) {
    const addressChanged = nextProps.account.address !== this.props.account.address;
    if (addressChanged) return true;

    const prevTokens = this.props.balance.tokens;
    const nextTokens = nextProps.balance.tokens;
    if (!prevTokens || !nextTokens) return true;
    if (prevTokens.length !== nextTokens.length) return true;

    const prevValues = prevTokens.map(t => `${t.token}:${t.value}`).sort();
    const nextValues = nextTokens.map(t => `${t.token}:${t.value}`).sort();

    return !isEqual(prevValues, nextValues);
  }

  render () {
    const { account, children, link, handleAddSearchToken } = this.props;
    const { tags } = account.meta;

    if (!account) {
      return null;
    }

    const { address } = account;
    const viewLink = `/${link || 'account'}/${address}`;

    const addressComponent = (
      <Input
        readOnly
        hideUnderline
        value={ address }
        allowCopy={ address }
      />
    );

    return (
      <Container>
        <Tags tags={ tags } handleAddSearchToken={ handleAddSearchToken } />

        <IdentityIcon address={ address } />

        <ContainerTitle
          title={ <Link to={ viewLink }>{ <IdentityName address={ address } unknown /> }</Link> }
          byline={ addressComponent }
        />

        { this.renderBalance() }

        { children }
      </Container>
    );
  }

  renderBalance () {
    const { balance } = this.props;

    if (!balance) {
      return null;
    }

    return (
      <Balance balances={ balance.tokens } />
    );
  }

}
