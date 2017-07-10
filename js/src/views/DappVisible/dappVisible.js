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
import { FormattedMessage } from 'react-intl';

import { DappCard, Page, SelectionList } from '@parity/ui';

import Store from './store';
import styles from './dappVisible.css';

@observer
export default class DappVisible extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  store = new Store(this.context.api);

  render () {
    return (
      <Page
        title={
          <FormattedMessage
            id='dapps.add.label'
            defaultMessage='visible applications'
          />
        }
      >
        <div className={ styles.warning } />
        {
          this.renderList(this.store.sortedLocal, this.store.displayApps,
            <FormattedMessage
              id='dapps.add.local.label'
              defaultMessage='Applications locally available'
            />,
            <FormattedMessage
              id='dapps.add.local.desc'
              defaultMessage='All applications installed locally on the machine by the user for access by the Parity client.'
            />
          )
        }
        {
          this.renderList(this.store.sortedBuiltin, this.store.displayApps,
            <FormattedMessage
              id='dapps.add.builtin.label'
              defaultMessage='Applications bundled with Parity'
            />,
            <FormattedMessage
              id='dapps.add.builtin.desc'
              defaultMessage='Experimental applications developed by the Parity team to show off dapp capabilities, integration, experimental features and to control certain network-wide client behaviour.'
            />
          )
        }
        {
          this.renderList(this.store.sortedNetwork, this.store.displayApps,
            <FormattedMessage
              id='dapps.add.network.label'
              defaultMessage='Applications on the global network'
            />,
            <FormattedMessage
              id='dapps.add.network.desc'
              defaultMessage='These applications are not affiliated with Parity nor are they published by Parity. Each remain under the control of their respective authors. Please ensure that you understand the goals for each application before interacting.'
            />
          )
        }
      </Page>
    );
  }

  renderList (items, visibleItems, header, byline) {
    if (!items || !items.length) {
      return null;
    }

    return (
      <div className={ styles.list }>
        <div className={ styles.background }>
          <div className={ styles.header }>{ header }</div>
          <div className={ styles.byline }>{ byline }</div>
        </div>
        <SelectionList
          isChecked={ this.isVisible }
          items={ items }
          noStretch
          onSelectClick={ this.onSelect }
          renderItem={ this.renderApp }
        />
      </div>
    );
  }

  renderApp = (app) => {
    return (
      <DappCard
        app={ app }
        key={ app.id }
      />
    );
  }

  isVisible = (app) => {
    return (this.store.displayApps[app.id] && this.store.displayApps[app.id].visible) || false;
  }

  onSelect = (app) => {
    if (this.isVisible(app)) {
      this.store.hideApp(app.id);
    } else {
      this.store.showApp(app.id);
    }
  }
}
