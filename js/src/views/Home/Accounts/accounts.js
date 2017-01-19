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

import moment from 'moment';
import React, { Component } from 'react';
import { FormattedMessage } from 'react-intl';
import { Link } from 'react-router';

import { Container, IdentityName, IdentityIcon } from '~/ui';
import { arrayOrObjectProptype } from '~/util/proptypes';

import styles from '../home.css';

export default class Accounts extends Component {
  static propTypes = {
    history: arrayOrObjectProptype().isRequired
  }

  render () {
    return (
      <Container
        title={
          <FormattedMessage
            id='home.accounts.title'
            defaultMessage='Recent Accounts'
          />
        }
      >
        <div className={ styles.accounts }>
          { this.renderHistory() }
        </div>
      </Container>
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

    const rows = history.map((h) => {
      return (
        <tr key={ h.timestamp }>
          <td className={ styles.timestamp }>
            { moment(h.timestamp).fromNow() }
          </td>
          <td className={ styles.entry }>
            <Link to={ `/accounts/${h.entry}` }>
              <IdentityIcon
                address={ h.entry }
                center
                className={ styles.identityIcon }
                inline
              />
              <IdentityName
                address={ h.entry }
                unknown
              />
            </Link>
          </td>
        </tr>
      );
    });

    return (
      <table className={ styles.history }>
        <tbody>
          { rows }
        </tbody>
      </table>
    );
  }
}
