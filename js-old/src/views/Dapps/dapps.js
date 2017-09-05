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

import { omitBy } from 'lodash';
import { Checkbox } from 'material-ui';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';

import { DappPermissions, DappsVisible } from '~/modals';
import PermissionStore from '~/modals/DappPermissions/store';
import { Actionbar, Button, DappCard, Page, SectionList } from '~/ui';
import { LockedIcon, RefreshIcon, VisibleIcon } from '~/ui/Icons';

import DappsStore from './dappsStore';

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
  permissionStore = new PermissionStore(this.context.api);

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
              onCheck={ this.onClickAcceptExternal }
            />
          </div>
        </div>
      );
    }

    return (
      <div>
        <DappPermissions permissionStore={ this.permissionStore } />
        <DappsVisible store={ this.store } />
        <Actionbar
          className={ styles.toolbar }
          title={
            <FormattedMessage
              id='dapps.label'
              defaultMessage='Decentralized Applications'
            />
          }
          buttons={ [
            <Button
              icon={ <RefreshIcon /> }
              key='refresh'
              label={
                <FormattedMessage
                  id='dapps.button.dapp.refresh'
                  defaultMessage='refresh'
                />
              }
              onClick={ this.store.refreshDapps }
            />,
            <Button
              icon={ <VisibleIcon /> }
              key='edit'
              label={
                <FormattedMessage
                  id='dapps.button.edit'
                  defaultMessage='edit'
                />
              }
              onClick={ this.store.openModal }
            />,
            <Button
              icon={ <LockedIcon /> }
              key='permissions'
              label={
                <FormattedMessage
                  id='dapps.button.permissions'
                  defaultMessage='permissions'
                />
              }
              onClick={ this.openPermissionsModal }
            />
          ] }
        />
        <Page>
          <div>{ this.renderList(this.store.visibleLocal) }</div>
          <div>{ this.renderList(this.store.visibleBuiltin) }</div>
          <div>{ this.renderList(this.store.visibleNetwork, externalOverlay) }</div>
        </Page>
      </div>
    );
  }

  renderList (items, overlay) {
    return (
      <SectionList
        items={ items }
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
        showTags
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
