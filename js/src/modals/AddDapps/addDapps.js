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

import { DappCard, Portal, SelectionList } from '~/ui';

import styles from './addDapps.css';

@observer
export default class AddDapps extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  };

  render () {
    const { store } = this.props;

    if (!store.modalOpen) {
      return null;
    }

    return (
      <Portal
        className={ styles.modal }
        onClose={ store.closeModal }
        open
        title={
          <FormattedMessage
            id='dapps.add.label'
            defaultMessage='visible applications'
          />
        }
      >
        <div className={ styles.warning } />
        {
          this.renderList(store.sortedLocal, store.displayApps,
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
          this.renderList(store.sortedBuiltin, store.displayApps,
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
          this.renderList(store.sortedNetwork, store.displayApps,
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
      </Portal>
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
    const { store } = this.props;

    return store.displayApps[app.id].visible;
  }

  onSelect = (app) => {
    const { store } = this.props;

    if (this.isVisible(app)) {
      store.hideApp(app.id);
    } else {
      store.showApp(app.id);
    }
  }
}
