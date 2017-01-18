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

import { Checkbox } from 'material-ui';
import { List, ListItem } from 'material-ui/List';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Modal, Button } from '~/ui';
import { DoneIcon } from '~/ui/Icons';

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
      <Modal
        actions={ [
          <Button
            icon={ <DoneIcon /> }
            key='done'
            label={
              <FormattedMessage
                id='dapps.add.button.done'
                defaultMessage='Done'
              />
            }
            onClick={ store.closeModal }
          />
        ] }
        compact
        title={
          <FormattedMessage
            id='dapps.add.label'
            defaultMessage='visible applications'
          />
        }
        visible
      >
        <div className={ styles.warning } />
        {
          this.renderList(store.sortedLocal,
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
          this.renderList(store.sortedBuiltin,
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
          this.renderList(store.sortedNetwork,
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
      </Modal>
    );
  }

  renderList (items, header, byline) {
    if (!items || !items.length) {
      return null;
    }

    return (
      <div className={ styles.list }>
        <div className={ styles.background }>
          <div className={ styles.header }>{ header }</div>
          <div className={ styles.byline }>{ byline }</div>
        </div>
        <List>
          { items.map(this.renderApp) }
        </List>
      </div>
    );
  }

  renderApp = (app) => {
    const { store } = this.props;
    const isHidden = !store.displayApps[app.id].visible;

    const onCheck = () => {
      if (isHidden) {
        store.showApp(app.id);
      } else {
        store.hideApp(app.id);
      }
    };

    return (
      <ListItem
        key={ app.id }
        leftCheckbox={
          <Checkbox
            checked={ !isHidden }
            onCheck={ onCheck }
          />
        }
        primaryText={ app.name }
        secondaryText={
          <div className={ styles.description }>
            { app.description }
          </div>
        }
      />
    );
  }
}
