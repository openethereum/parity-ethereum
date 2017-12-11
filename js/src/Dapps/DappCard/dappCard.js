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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { Link } from 'react-router';

import DappIcon from '@parity/ui/lib/DappIcon';
import Header from 'semantic-ui-react/dist/commonjs/elements/Header';
import Button from 'semantic-ui-react/dist/commonjs/elements/Button';

import styles from './dappCard.css';

export default class DappCard extends Component {
  static propTypes = {
    app: PropTypes.object.isRequired,
    availability: PropTypes.string.isRequired,
    className: PropTypes.string,
    onPin: PropTypes.func,
    pinned: PropTypes.bool
  };

  handlePin = () => this.props.onPin(this.props.app.id)

  render () {
    const { app, availability, className, pinned } = this.props;

    if (app.onlyPersonal && availability !== 'personal') {
      return null;
    }

    return (
      <div className={ [styles.card, className].join(' ') }>
        <Button
          size='mini'
          icon='pin'
          circular
          className={ [styles.pin, pinned && styles.pinned].join(' ') }
          onClick={ this.handlePin }
        />
        <div className={ styles.content }>
          <Link to={ app.url === 'web' ? '/web' : `/${app.id}` } >
            <DappIcon
              app={ app }
              className={ styles.image }
            />
            <Header
              as='h5'
              textAlign='center'
              className={ styles.title }
            >
              {app.name}
            </Header>
          </Link>
        </div>
      </div>
    );
  }
}
