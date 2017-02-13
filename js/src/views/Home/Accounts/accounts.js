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
import React, { Component } from 'react';
import { FormattedMessage } from 'react-intl';
import { Link } from 'react-router';

import { Container, ContainerTitle, IdentityName, IdentityIcon, SectionList } from '~/ui';
import { arrayOrObjectProptype } from '~/util/proptypes';

import styles from './accounts.css';

export default class Accounts extends Component {
  static propTypes = {
    history: arrayOrObjectProptype().isRequired
  }

  render () {
    return (
      <div className={ styles.accounts }>
        <ContainerTitle
          title={
            <FormattedMessage
              id='home.accounts.title'
              defaultMessage='Recent Accounts'
            />
          }
        />
        { this.renderHistory() }
      </div>
    );
  }

  renderHistory () {
    const { history } = this.props;

    if (!history.length) {
      return (
        <div className={ styles.empty }>
          <FormattedMessage
            id='home.accounts.none'
            defaultMessage='No recent accounts history available'
          />
        </div>
      );
    }

    return (
      <SectionList
        items={ history }
        renderItem={ this.renderHistoryItem }
      />
    );
  }

  renderHistoryItem = (history) => {
    if (!history || !history.entry) {
      return null;
    }

    return (
      <Container
        className={ styles.account }
        key={ history.timestamp }
        hover={
          <div className={ styles.timestamp }>
            <FormattedMessage
              id='home.account.visited'
              defaultMessage='accessed {when}'
              values={ {
                when: moment(history.timestamp).fromNow()
              } }
            />
          </div>
        }
      >
        <Link
          className={ styles.link }
          to={ `/${history.type === 'wallet' ? 'wallet' : 'accounts'}/${history.entry}` }
        >
          <IdentityIcon
            address={ history.entry }
            className={ styles.icon }
            center
          />
          <IdentityName
            address={ history.entry }
            className={ styles.name }
            unknown
          />
        </Link>
      </Container>
    );
  }
}
