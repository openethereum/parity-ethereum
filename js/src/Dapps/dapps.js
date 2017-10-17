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

import omitBy from 'lodash.omitby';
import { observer } from 'mobx-react';
import React, { Component } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import PropTypes from 'prop-types';

import DappCard from '@parity/ui/DappCard';
import Checkbox from '@parity/ui/Form/Checkbox';
import Page from '@parity/ui/Page';
import SectionList from '@parity/ui/SectionList';

import DappsStore from '@parity/shared/mobx/dappsStore';

import styles from './dapps.css';

@observer
class Dapps extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    availability: PropTypes.string.isRequired
  };

  store = DappsStore.get(this.context.api);

  componentWillMount () {
    this.store.loadAllApps();
  }

  render () {
    let externalOverlay = null;

    if (this.store.externalOverlayVisible) {
      externalOverlay = (
        <div className={ styles.overlay }>
          <div>
            <FormattedMessage
              id='dapps.external.warning'
              defaultMessage='Applications made available on the network by 3rd-party authors are not affiliated with Parity nor are they published by Parity. Each remain under the control of their respective authors. Please ensure that you understand the goals for each before interacting.'
            />
          </div>
          <div>
            <Checkbox
              className={ styles.accept }
              label={
                <FormattedMessage
                  id='dapps.external.accept'
                  defaultMessage='I understand that these applications are not affiliated with Parity'
                />
              }
              checked={ false }
              onClick={ this.onClickAcceptExternal }
            />
          </div>
        </div>
      );
    }

    return (
      <Page
        title={
          <FormattedMessage
            id='dapps.label'
            defaultMessage='Decentralized Applications'
          />
        }
      >
        { this.renderList(this.store.visibleViews) }
        { this.renderList(this.store.visibleLocal) }
        { this.renderList(this.store.visibleBuiltin) }
        { this.renderList(this.store.visibleNetwork, externalOverlay) }
      </Page>
    );
  }

  renderList (items, overlay) {
    console.log(items);
    return (
      <SectionList
        items={ items }
        noStretch
        overlay={ overlay }
        renderItem={ this.renderApp }
      />
    );
  }

  renderApp = (app) => {
    if (app.onlyPersonal && this.props.availability !== 'personal') {
      return null;
    }

    return (
      <DappCard
        app={ app }
        key={ app.id }
        showLink
      />
    );
  }

  onClickAcceptExternal = () => {
    this.store.closeExternalOverlay();
  }

  openPermissionsModal = () => {
    const { accounts } = this.props;

    this.permissionStore.openModal(accounts);
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;
  const { availability = 'unknown' } = state.nodeStatus.nodeKind || {};

  /**
   * Do not show the Wallet Accounts in the Dapps
   * Permissions Modal. This will come in v1.6, but
   * for now it would break dApps using Web3...
   */
  const _accounts = omitBy(accounts, (account) => account.wallet);

  return {
    accounts: _accounts,
    availability
  };
}

export default connect(
  mapStateToProps,
  null
)(Dapps);
