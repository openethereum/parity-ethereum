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
import DoneIcon from 'material-ui/svg-icons/action/done';
import { List, ListItem } from 'material-ui/List';
import Checkbox from 'material-ui/Checkbox';

import { Modal, Button } from '../../../ui';

import styles from './AddDapps.css';

export default class AddDapps extends Component {
  static propTypes = {
    available: PropTypes.array.isRequired,
    visible: PropTypes.array.isRequired,
    open: PropTypes.bool.isRequired,
    onAdd: PropTypes.func.isRequired,
    onRemove: PropTypes.func.isRequired,
    onClose: PropTypes.func.isRequired
  };

  render () {
    const { onClose, open, available } = this.props;

    return (
      <Modal
        title='Select Distributed Apps to be shown'
        actions={ [
          <Button label={ 'Done' } onClick={ onClose } icon={ <DoneIcon /> } />
        ] }
        visible={ open }
        scroll
      >
        <List>
          { available.map(this.renderApp) }
        </List>
      </Modal>
    );
  }

  renderApp = (app) => {
    const { visible, onAdd, onRemove } = this.props;
    const isVisible = visible.includes(app.id);
    const onCheck = () => {
      if (isVisible) onRemove(app.id);
      else onAdd(app.id);
    };

    return (
      <ListItem
        key={ app.id }
        leftCheckbox={ <Checkbox checked={ isVisible } onCheck={ onCheck } /> }
        primaryText={ app.name }
        secondaryText={ <pre className={ styles.hash }><code>{ app.hash }</code></pre> }
      />
    );
  }
}
