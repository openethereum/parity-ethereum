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
import { Link } from 'react-router';

import { Container, DappUrlInput, IdentityName, IdentityIcon, Page } from '~/ui';

import DappsStore from '../Dapps/dappsStore';
import HistoryStore from '../historyStore';
import WebStore from '../Web/store';
import styles from './home.css';

@observer
export default class Home extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    router: PropTypes.object.isRequired
  };

  dappsStore = DappsStore.get(this.context.api);
  webStore = WebStore.get(this.context.api);

  accountsHistory = HistoryStore.get('accounts');
  dappsHistory = HistoryStore.get('dapps');
  webHistory = HistoryStore.get('web');

  state = {
    dapps: {}
  };

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
          { this.renderAccountsHistory() }
        </div>
      </Container>
    );
  }

  renderAccountsHistory () {
    const { history } = this.accountsHistory;

    if (!history.length) {
      return (
        <div className={ styles.empty }>
          No recent accounts retrieved
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
          { this.renderDappsHistory() }
        </div>
      </Container>
    );
  }

  renderDappsHistory () {
    const { dapps } = this.state;
    const { history } = this.dappsHistory;

    if (!history.length) {
      return (
        <div className={ styles.empty }>
          No recent applications retrieved
        </div>
      );
    }

    const rows = history.map((h) => {
      const dapp = dapps[h.entry];

      if (typeof dapp === 'undefined') {
        this.loadApp(h.entry);
      }

      if (!dapp) {
        return null;
      }

      return (
        <tr key={ h.timestamp }>
          <td className={ styles.timestamp }>
            { moment(h.timestamp).fromNow() }
          </td>
          <td className={ styles.entry }>
            <Link to={ `/app/${h.entry}` }>
              <img
                className={ styles.dappIcon }
                src={ '' }
              />
              <span>
                { dapp.name || h.entry }
              </span>
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

  renderUrl () {
    const { nextUrl } = this.webStore;

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
      return (
        <div className={ styles.empty }>
          No recent URLs available
        </div>
      );
    }

    const rows = history.map((h) => {
      const onNavigate = () => this.onGotoUrl(h.entry);

      return (
        <tr key={ h.timestamp }>
          <td className={ styles.timestamp }>
            { moment(h.timestamp).fromNow() }
          </td>
          <td className={ styles.entry }>
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
    this.webStore.setNextUrl(url);
  }

  onGotoUrl = (url) => {
    const { router } = this.context;

    this.webStore.gotoUrl(url);
    router.push('/web');
  }

  onRestoreUrl = () => {
    this.webStore.restoreUrl();
  }

  loadApp = (id) => {
    const { dapps } = this.state;

    if (dapps[id]) {
      return;
    }

    this.dappsStore
      .loadApp(id)
      .then((app) => {
        console.log(id, app);
        this.setState({
          dapps: Object.assign({ ...this.state.dapps }, { [id]: app })
        });
      })
      .catch((error) => {
        console.warn(`Unable to load ${id}`, error);
      });
  }
}
