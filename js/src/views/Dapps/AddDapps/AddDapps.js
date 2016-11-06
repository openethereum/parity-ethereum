// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import { observer } from 'mobx-react';
import DoneIcon from 'material-ui/svg-icons/action/done';
import { List, ListItem } from 'material-ui/List';
import Checkbox from 'material-ui/Checkbox';

import { Modal, Button } from '../../../ui';

import styles from './AddDapps.css';

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
        compact
        title='visible applications'
        actions={ [
          <Button
            label={ 'Done' }
            key='done'
            onClick={ store.closeModal }
            icon={ <DoneIcon /> }
          />
        ] }
        visible
        scroll>
        <List>
          { store.apps.map(this.renderApp) }
        </List>
      </Modal>
    );
  }

  renderApp = (app) => {
    const { store } = this.props;
    const isHidden = store.hidden.includes(app.id);
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
