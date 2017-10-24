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

import moment from 'moment';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { Link } from 'react-router';

import { Container, DappIcon } from '~/ui';

import styles from './dapps.css';

export default class Dapp extends Component {
  static propTypes = {
    id: PropTypes.string.isRequired,
    store: PropTypes.object.isRequired,
    timestamp: PropTypes.number.isRequired
  }

  state = {
    dapp: null
  }

  componentWillMount () {
    this.isInactive = false;
    return this.loadApp();
  }

  componentWillUnmount () {
    this.isInactive = true;
  }

  render () {
    const { id, timestamp } = this.props;
    const { dapp } = this.state;

    if (!dapp) {
      return null;
    }

    return (
      <Container
        className={ styles.dapp }
        hover={
          <div className={ styles.timestamp }>
            <FormattedMessage
              id='home.dapp.visited'
              defaultMessage='accessed {when}'
              values={ {
                when: moment(timestamp).fromNow()
              } }
            />
          </div>
        }
      >
        <Link
          className={ styles.link }
          to={ `/app/${id}` }
        >
          <DappIcon
            app={ dapp }
            className={ styles.icon }
          />
          <span className={ styles.name }>
            { dapp.name }
          </span>
        </Link>
      </Container>
    );
  }

  loadApp = () => {
    const { id, store } = this.props;

    return store
      .loadApp(id)
      .then((dapp) => {
        if (this.isInactive) {
          return;
        }

        this.setState({ dapp });
      });
  }
}
