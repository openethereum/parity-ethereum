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

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { Checkbox } from 'material-ui';
import { observer } from 'mobx-react';

import { Actionbar, Page } from '~/ui';
import FlatButton from 'material-ui/FlatButton';
import EyeIcon from 'material-ui/svg-icons/image/remove-red-eye';

import DappsStore from './dappsStore';

import AddDapps from './AddDapps';
import Summary from './Summary';

import styles from './dapps.css';

@observer
export default class Dapps extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  store = DappsStore.get(this.context.api);

  render () {
    let externalOverlay = null;
    if (this.store.externalOverlayVisible) {
      externalOverlay = (
        <div className={ styles.overlay }>
          <div className={ styles.body }>
            <div>
              <FormattedMessage
                id='dapps.external.warning'
                defaultMessage='Applications made available on the network by 3rd-party authors are not affiliated with Parity nor are they published by Parity. Each remain under the control of their respective authors. Please ensure that you understand the goals for each before interacting.' />
            </div>
            <div>
              <Checkbox
                className={ styles.accept }
                label={
                  <FormattedMessage
                    id='dapps.external.accept'
                    defaultMessage='I understand that these applications are not affiliated with Parity' />
                }
                checked={ false }
                onCheck={ this.onClickAcceptExternal } />
            </div>
          </div>
        </div>
      );
    }

    return (
      <div>
        <AddDapps store={ this.store } />
        <Actionbar
          className={ styles.toolbar }
          title={
            <FormattedMessage
              id='dapps.label'
              defaultMessage='Decentralized Applications' />
          }
          buttons={ [
            <FlatButton
              label={
                <FormattedMessage
                  id='dapps.button.edit'
                  defaultMessage='edit' />
              }
              key='edit'
              icon={ <EyeIcon /> }
              onTouchTap={ this.store.openModal }
            />
          ] }
        />
        <Page>
          <div>
            { this.renderList(this.store.visibleLocal) }
          </div>

          <div>
            { this.renderList(this.store.visibleBuiltin) }
          </div>

          <div>
            { this.renderList(this.store.visibleNetwork, externalOverlay) }
          </div>
        </Page>
      </div>
    );
  }

  renderList (items, overlay) {
    if (!items || !items.length) {
      return null;
    }

    return (
      <div className={ styles.list }>
        { overlay }
        { items.map(this.renderApp) }
      </div>
    );
  }

  renderApp = (app) => {
    return (
      <div
        className={ styles.item }
        key={ app.id }>
        <Summary app={ app } />
      </div>
    );
  }

  onClickAcceptExternal = () => {
    this.store.closeExternalOverlay();
  }
}
