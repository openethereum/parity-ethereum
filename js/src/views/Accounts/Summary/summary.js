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

import { Balance, Container, ContainerTitle, IdentityIcon, IdentityName, Tags, Input } from '../../../ui';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  };

  static propTypes = {
    account: PropTypes.object.isRequired,
    balance: PropTypes.object,
    link: PropTypes.string,
    name: PropTypes.string,
    noLink: PropTypes.bool,
    children: PropTypes.node,
    handleAddSearchToken: PropTypes.func
  };

  static defaultProps = {
    noLink: false
  };

  state = {
    name: 'Unnamed'
  };

  render () {
    const { account, children, handleAddSearchToken } = this.props;
    const { tags } = account.meta;

    if (!account) {
      return null;
    }

    const { address } = account;

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
        <IdentityIcon
          address={ address } />
        <ContainerTitle
          title={ this.renderLink() }
          byline={ addressComponent } />

        { this.renderBalance() }
        { children }
      </Container>
    );
  }

  renderLink () {
    const { link, noLink, account, name } = this.props;

    const { address } = account;
    const viewLink = `/${link || 'account'}/${address}`;

    const content = (
      <IdentityName address={ address } name={ name } unknown />
    );

    if (noLink) {
      return content;
    }

    return (
      <Link to={ viewLink }>
        { content }
      </Link>
    );
  }

  renderBalance () {
    const { balance } = this.props;

    if (!balance) {
      return null;
    }

    return (
      <Balance balance={ balance } />
    );
  }
}
