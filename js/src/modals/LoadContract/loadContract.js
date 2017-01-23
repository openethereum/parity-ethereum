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

import ContentClear from 'material-ui/svg-icons/content/clear';
import CheckIcon from 'material-ui/svg-icons/navigation/check';
import DeleteIcon from 'material-ui/svg-icons/action/delete';

import { List, ListItem, makeSelectable } from 'material-ui/List';
import { Subheader, IconButton, Tabs, Tab } from 'material-ui';
import moment from 'moment';

import { Button, Modal, Editor } from '~/ui';

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
    contracts: PropTypes.object.isRequired,
    snippets: PropTypes.object.isRequired
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

    const { contracts, snippets } = this.props;

    const contractsTab = Object.keys(contracts).length === 0
      ? null
      : (
        <Tab label='Local' >
          { this.renderEditor() }

          <SelectableList
            onChange={ this.onClickContract }
          >
            <Subheader>Saved Contracts</Subheader>
            { this.renderContracts(contracts) }
          </SelectableList>
        </Tab>
      );

    return (
      <div className={ styles.loadContainer }>
        <Tabs onChange={ this.handleChangeTab }>
          { contractsTab }

          <Tab label='Snippets' >
            { this.renderEditor() }

            <SelectableList
              onChange={ this.onClickContract }
            >
              <Subheader>Contract Snippets</Subheader>
              { this.renderContracts(snippets, false) }
            </SelectableList>
          </Tab>
        </Tabs>
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
    const { contracts, snippets } = this.props;
    const { selected } = this.state;

    const mergedContracts = Object.assign({}, contracts, snippets);

    if (selected === -1 || !mergedContracts[selected]) {
      return null;
    }

    const { sourcecode, name } = mergedContracts[selected];

    return (
      <div className={ styles.editor }>
        <p>{ name }</p>
        <Editor
          value={ sourcecode }
          maxLines={ 20 }
          readOnly
        />
      </div>
    );
  }

  renderContracts (contracts, removable = true) {
    const { selected } = this.state;

    return Object
      .values(contracts)
      .map((contract) => {
        const { id, name, timestamp, description } = contract;
        const onDelete = () => this.onDeleteRequest(id);

        const secondaryText = description || `Saved ${moment(timestamp).fromNow()}`;
        const remove = removable
          ? (
            <IconButton onTouchTap={ onDelete }>
              <DeleteIcon />
            </IconButton>
          )
          : null;

        return (
          <ListItem
            value={ id }
            key={ id }
            primaryText={ name }
            secondaryText={ secondaryText }
            style={ selected === id ? SELECTED_STYLE : null }
            rightIconButton={ remove }
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

  handleChangeTab = () => {
    this.setState({ selected: -1 });
  }

  onClickContract = (_, value) => {
    this.setState({ selected: value });
  }

  onClose = () => {
    this.props.onClose();
  }

  onLoad = () => {
    const { contracts, snippets } = this.props;
    const { selected } = this.state;

    const mergedContracts = Object.assign({}, contracts, snippets);
    const contract = mergedContracts[selected];

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
