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

import Container, { Title } from '../../../ui/Container';
import IdentityIcon from '../../../ui/IdentityIcon';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    app: PropTypes.object.isRequired,
    tokens: PropTypes.object,
    children: PropTypes.node
  }

  render () {
    const { app } = this.props;

    if (!app) {
      return null;
    }

    const url = `/app/${app.url}`;

    return (
      <Container>
        <IdentityIcon
          address={ app.id } />
        <Title
          title={ <Link to={ url }>{ app.name }</Link> }
          byline={ app.description } />
        { this.props.children }
      </Container>
    );
  }
}
