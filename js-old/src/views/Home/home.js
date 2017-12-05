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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { FormattedMessage } from 'react-intl';

import HistoryStore from '~/mobx/historyStore';
import { Page } from '~/ui';

import DappsStore from '../Dapps/dappsStore';
import ExtensionStore from '../Application/Extension/store';
import WebStore from '../Web/store';

import Accounts from './Accounts';
import Dapps from './Dapps';
import News from './News';
import Urls from './Urls';
import styles from './home.css';

@observer
class Home extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    availability: PropTypes.string.isRequired
  };

  dappsStore = DappsStore.get(this.context.api);
  extensionStore = ExtensionStore.get();
  webStore = WebStore.get(this.context.api);

  accountsHistory = HistoryStore.get('accounts');
  dappsHistory = HistoryStore.get('dapps');

  componentWillMount () {
    return this.webStore.loadHistory();
  }

  render () {
    const urls = this.props.availability !== 'personal' ? null : (
      <Urls
        extensionStore={ this.extensionStore }
        store={ this.webStore }
      />
    );

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
        <News />
        { urls }
        <div className={ styles.row }>
          <div className={ styles.column }>
            <Dapps
              history={ this.dappsHistory.history }
              store={ this.dappsStore }
            />
          </div>
          <div className={ styles.column }>
            <Accounts history={ this.accountsHistory.history } />
          </div>
        </div>
      </Page>
    );
  }
}

function mapStateToProps (initState) {
  return (state) => {
    const { availability = 'unknown' } = state.nodeStatus.nodeKind || {};

    return { availability };
  };
}

export default connect(
  mapStateToProps,
  null
)(Home);
