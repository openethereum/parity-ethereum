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
import { Link } from 'react-router';

import Container, { Title as ContainerTitle } from '~/ui/Container';
import DappIcon from '~/ui/DappIcon';
import Tags from '~/ui/Tags';

import styles from './dappCard.css';

export default class DappCard extends Component {
  static propTypes = {
    app: PropTypes.object.isRequired,
    children: PropTypes.node,
    className: PropTypes.string,
    onClick: PropTypes.func,
    showLink: PropTypes.bool,
    showTags: PropTypes.bool
  };

  static defaultProps = {
    showLink: false,
    showTags: false
  };

  render () {
    const { app, children, className, onClick, showLink, showTags } = this.props;

    if (!app) {
      return null;
    }

    return (
      <Container
        className={
          [styles.container, className].join(' ')
        }
        onClick={ onClick }
      >
        <DappIcon
          app={ app }
          className={ styles.image }
        />
        <Tags
          tags={
            showTags
              ? [app.type]
              : null
          }
        />
        <div className={ styles.description }>
          <ContainerTitle
            className={ styles.title }
            title={
              showLink
                ? this.renderLink(app)
                : app.name
            }
            byline={ app.description }
          />
          <div className={ styles.author }>
            { app.author }, v{ app.version }
          </div>
          { children }
        </div>
      </Container>
    );
  }

  renderLink (app) {
    return (
      <Link
        to={
          app.url === 'web'
            ? '/web'
            : `/app/${app.id}`
        }
      >
        { app.name }
      </Link>
    );
  }
}
