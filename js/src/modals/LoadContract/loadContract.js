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

import ContentClear from 'material-ui/svg-icons/content/clear';
import CheckIcon from 'material-ui/svg-icons/navigation/check';
import DeleteIcon from 'material-ui/svg-icons/action/delete';

import { List, ListItem, makeSelectable } from 'material-ui/List';
import { Subheader, IconButton } from 'material-ui';
import moment from 'moment';

import { Button, Modal, Editor } from '../../ui';

import styles from './loadContract.css';

const SelectableList = makeSelectable(List);

const SELECTED_STYLE = {
  backgroundColor: 'rgba(255, 255, 255, 0.1)'
};

export default class LoadContract extends Component {

  static propTypes = {
    onClose: PropTypes.func.isRequired,
    onLoad: PropTypes.func.isRequired,
    onDelete: PropTypes.func.isRequired,
    contracts: PropTypes.object.isRequired
  };

  state = {
    selected: -1,
    deleteRequest: false,
    deleteId: -1
  };

  render () {
    const { deleteRequest } = this.state;

    const title = deleteRequest
      ? 'confirm removal'
      : 'view contracts';

    return (
      <Modal
        title={ title }
        actions={ this.renderDialogActions() }
        visible
      >
        { this.renderBody() }
      </Modal>
    );
  }

  renderBody () {
    if (this.state.deleteRequest) {
      return this.renderConfirmRemoval();
    }

    return (
      <div className={ styles.loadContainer }>
        <SelectableList
          onChange={ this.onClickContract }
        >
          <Subheader>Saved Contracts</Subheader>
          { this.renderContracts() }
        </SelectableList>

        { this.renderEditor() }
      </div>
    );
  }

  renderConfirmRemoval () {
    const { deleteId } = this.state;
    const { name, timestamp, sourcecode } = this.props.contracts[deleteId];

    return (
      <div className={ styles.confirmRemoval }>
        <p>
          Are you sure you want to remove the following
          contract from your saved contracts?
        </p>
        <ListItem
          primaryText={ name }
          secondaryText={ `Saved ${moment(timestamp).fromNow()}` }
          style={ { backgroundColor: 'none', cursor: 'default' } }
        />

        <div className={ styles.editor }>
          <Editor
            value={ sourcecode }
            maxLines={ 20 }
            readOnly
          />
        </div>
      </div>
    );
  }

  renderEditor () {
    const { contracts } = this.props;
    const { selected } = this.state;

    if (selected === -1 || !contracts[selected]) {
      return null;
    }

    const { sourcecode, name } = contracts[selected];

    return (
      <div className={ styles.editor }>
        <p>{ name }</p>
        <Editor
          value={ sourcecode }
          readOnly
        />
      </div>
    );
  }

  renderContracts () {
    const { contracts } = this.props;
    const { selected } = this.state;

    return Object
      .values(contracts)
      .map((contract) => {
        const { id, name, timestamp } = contract;
        const onDelete = () => this.onDeleteRequest(id);

        return (
          <ListItem
            value={ id }
            key={ id }
            primaryText={ name }
            secondaryText={ `Saved ${moment(timestamp).fromNow()}` }
            style={ selected === id ? SELECTED_STYLE : null }
            rightIconButton={ (
              <IconButton onClick={ onDelete }>
                <DeleteIcon />
              </IconButton>
            ) }
          />
        );
      });
  }

  renderDialogActions () {
    const { deleteRequest } = this.state;

    if (deleteRequest) {
      return [
        <Button
          icon={ <ContentClear /> }
          label='No'
          key='No'
          onClick={ this.onRejectRemoval }
        />,
        <Button
          icon={ <DeleteIcon /> }
          label='Yes'
          key='Yes'
          onClick={ this.onConfirmRemoval }
        />
      ];
    }

    const cancelBtn = (
      <Button
        icon={ <ContentClear /> }
        label='Cancel'
        onClick={ this.onClose }
      />
    );

    const loadBtn = (
      <Button
        icon={ <CheckIcon /> }
        label='Load'
        onClick={ this.onLoad }
        disabled={ this.state.selected === -1 }
      />
    );

    return [ cancelBtn, loadBtn ];
  }

  onClickContract = (_, value) => {
    this.setState({ selected: value });
  }

  onClose = () => {
    this.props.onClose();
  }

  onLoad = () => {
    const contract = this.props.contracts[this.state.selected];

    this.props.onLoad(contract);
    this.props.onClose();
  }

  onDeleteRequest = (id) => {
    this.setState({
      deleteRequest: true,
      deleteId: id
    });
  }

  onConfirmRemoval = () => {
    const { deleteId } = this.state;
    this.props.onDelete(deleteId);

    this.setState({
      deleteRequest: false,
      deleteId: -1,
      selected: -1
    });
  }

  onRejectRemoval = () => {
    this.setState({
      deleteRequest: false,
      deleteId: -1
    });
  }

}
