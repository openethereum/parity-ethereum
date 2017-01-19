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

import { Container, DappUrlInput } from '~/ui';
import { arrayOrObjectProptype } from '~/util/proptypes';

import styles from '../home.css';

@observer
export default class Urls extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  };

  static propTypes = {
    history: arrayOrObjectProptype().isRequired,
    store: PropTypes.object.isRequired
  }

  render () {
    const { nextUrl } = this.props.store;

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
            id='home.url.none'
            defaultMessage='No recent URLs available'
          />
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
    this.props.store.setNextUrl(url);
  }

  onGotoUrl = (url) => {
    const { router } = this.context;

    this.props.store.gotoUrl(url);
    router.push('/web');
  }

  onRestoreUrl = () => {
    this.props.store.restoreUrl();
  }
}
