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

import { Container, ContainerTitle } from '../../../ui';

import styles from './summary.css';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    app: PropTypes.object.isRequired,
    children: PropTypes.node
  }

  render () {
    const { dappsPort } = this.context.api;
    const { app } = this.props;

    if (!app) {
      return null;
    }

    let image = <div className={ styles.image }>&nbsp;</div>;
    if (app.type === 'local') {
      image = <img src={ `http://127.0.0.1:${dappsPort}/${app.id}/${app.iconUrl}` } className={ styles.image } />;
    } else {
      image = <img src={ `http://127.0.0.1:${dappsPort}${app.image}` } className={ styles.image } />;
    }

    return (
      <Container className={ styles.container }>
        { image }
        <div className={ styles.description }>
          <ContainerTitle
            className={ styles.title }
            title={
              <Link to={ `/app/${app.id}` }>
                { app.name }
              </Link>
            }
            byline={ app.description }
          />
          <div className={ styles.author }>
            { app.author }, v{ app.version }
          </div>
          { this.props.children }
        </div>
      </Container>
    );
  }
}
