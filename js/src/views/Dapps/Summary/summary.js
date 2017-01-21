// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { Container, ContainerTitle, Tags } from '~/ui';

import styles from '../dapps.css';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    app: PropTypes.object.isRequired,
    children: PropTypes.node
  }

  render () {
    const { dappsUrl } = this.context.api;
    const { app } = this.props;

    if (!app) {
      return null;
    }

    return (
      <Link
        to={
          app.url === 'web'
            ? '/web'
            : `/app/${app.id}`
        }
      >
        <Container className={ styles.summary }>
          <img
            className={ styles.image }
            src={
              app.type === 'local'
                ? `${dappsUrl}/${app.id}/${app.iconUrl}`
                : `${dappsUrl}${app.image}`
            }
          />
          <Tags
            className={ styles.tags }
            tags={ [app.type] }
          />
          <div className={ styles.description }>
            <ContainerTitle
              className={ styles.title }
              title={ app.name }
              byline={ app.description }
            />
            <div className={ styles.author }>
              { app.author }, v{ app.version }
            </div>
            { this.props.children }
          </div>
        </Container>
      </Link>
    );
  }
}
