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
import { connect } from 'react-redux';
import { Link } from 'react-router';

import { Container, ContainerTitle, IdentityName, IdentityIcon, SectionList } from '~/ui';
import { arrayOrObjectProptype } from '~/util/proptypes';

import styles from './accounts.css';

class Accounts extends Component {
  static propTypes = {
    accountsInfo: PropTypes.object,
    history: arrayOrObjectProptype().isRequired
  };

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
    const { accountsInfo, history } = this.props;

    if (!accountsInfo || !Object.keys(accountsInfo).length) {
      return null;
    }

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
    const { accountsInfo } = this.props;

    if (!history || !history.entry) {
      return null;
    }

    const account = accountsInfo[history.entry];

    if (!account) {
      return null;
    }

    let linkType = 'addresses';

    if (account.uuid) {
      linkType = 'accounts';
    } else if (account.meta.wallet) {
      linkType = 'wallet';
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
          to={ `/${linkType}/${history.entry}` }
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

function mapStateToProps (state) {
  const { accountsInfo } = state.personal;

  return {
    accountsInfo
  };
}

export default connect(
  mapStateToProps,
  null
)(Accounts);
