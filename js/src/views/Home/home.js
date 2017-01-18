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

import { observer } from 'mobx-react';
import moment from 'moment';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Container, DappUrlInput, Page } from '~/ui';

import HistoryStore from '../historyStore';
import WebStore from '../Web/store';
import styles from './home.css';

@observer
export default class Home extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    router: PropTypes.object.isRequired
  };

  webstore = WebStore.get(this.context.api);
  webHistory = HistoryStore.get('web');

  render () {
    return (
      <Page
        className={ styles.body }
        title={
          <FormattedMessage
            id='home.title'
            defaultMessage='Parity Home'
          />
        }
      >
        <div className={ styles.list }>
          <div className={ styles.item }>
            { this.renderUrl() }
          </div>
          <div className={ styles.item }>
            { this.renderDapps() }
          </div>
          <div className={ styles.item }>
            { this.renderAccounts() }
          </div>
        </div>
      </Page>
    );
  }

  renderAccounts () {
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
          Something goes in here
        </div>
      </Container>
    );
  }

  renderDapps () {
    return (
      <Container
        title={
          <FormattedMessage
            id='home.dapps.title'
            defaultMessage='Recent Dapps'
          />
        }
      >
        <div className={ styles.dapps }>
          Something goes in here
        </div>
      </Container>
    );
  }

  renderUrl () {
    const { nextUrl } = this.webstore;

    return (
      <Container
        title={
          <FormattedMessage
            id='home.url.title'
            defaultMessage='Web URLs'
          />
        }
      >
        <div className={ styles.urls }>
          <DappUrlInput
            className={ styles.input }
            onChange={ this.onChangeUrl }
            onGoto={ this.onGotoUrl }
            onRestore={ this.onRestoreUrl }
            url={ nextUrl }
          />
          { this.renderUrlHistory() }
        </div>
      </Container>
    );
  }

  renderUrlHistory () {
    const { history } = this.webHistory;

    if (!history.length) {
      return null;
    }

    const rows = history.map((h) => {
      const onNavigate = () => this.onGotoUrl(h.entry);

      return (
        <tr key={ h.timestamp }>
          <td className={ styles.timestamp }>
            { moment(h.timestamp).fromNow() }
          </td>
          <td className={ h.entry }>
            <a
              href='javascript:void(0)'
              onClick={ onNavigate }
            >
              { h.entry }
            </a>
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

  onChangeUrl = (url) => {
    this.webstore.setNextUrl(url);
  }

  onGotoUrl = (url) => {
    const { router } = this.context;

    this.webstore.gotoUrl(url);
    router.push('/web');
  }

  onRestoreUrl = () => {
    this.webstore.restoreUrl();
  }
}
