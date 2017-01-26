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

import React, { Component, PropTypes } from 'react';
import { Dialog, RaisedButton, FlatButton, SelectField, MenuItem } from 'material-ui';
import AddIcon from 'material-ui/svg-icons/content/add';

import InputText from '../../Inputs/Text';
import { ADDRESS_TYPE } from '../../Inputs/validation';

import styles from './token.css';

import { metaDataKeys } from '../../constants';

const initState = {
  showDialog: false,
  complete: false,
  metaKeyIndex: 0,

  form: {
    valid: false,
    value: ''
  }
};

export default class AddMeta extends Component {
  static propTypes = {
    isTokenOwner: PropTypes.bool,
    handleAddMeta: PropTypes.func,
    index: PropTypes.number
  };

  state = initState;

  render () {
    if (!this.props.isTokenOwner) {
      return null;
    }

    return (<div className={ styles['add-meta'] }>
      <RaisedButton
        label='Add Meta-Data'
        icon={ <AddIcon /> }
        primary
        fullWidth
        onTouchTap={ this.onShowDialog }
      />

      <Dialog
        title='add meta data'
        open={ this.state.showDialog }
        modal={ this.state.complete }
        className={ styles.dialog }
        onRequestClose={ this.onClose }
        actions={ this.renderActions() }
      >
        { this.renderContent() }
      </Dialog>
    </div>);
  }

  renderActions () {
    const { complete } = this.state;

    if (complete) {
      return (
        <FlatButton
          label='Done'
          primary
          onTouchTap={ this.onClose }
        />
      );
    }

    const isValid = this.state.form.valid;

    return ([
      <FlatButton
        label='Cancel'
        primary
        onTouchTap={ this.onClose }
      />,
      <FlatButton
        label='Add'
        primary
        disabled={ !isValid }
        onTouchTap={ this.onAdd }
      />
    ]);
  }

  renderContent () {
    const { complete } = this.state;

    if (complete) {
      return this.renderComplete();
    }

    return this.renderForm();
  }

  renderComplete () {
    if (metaDataKeys[this.state.metaKeyIndex].value === 'IMG') {
      return (<div>
        <p>
        Your transactions has been posted.
        Two transactions are needed to add an Image.
        Please visit the Parity Signer to authenticate the transfer.</p>
      </div>);
    }
    return (<div>
      <p>Your transaction has been posted. Please visit the Parity Signer to authenticate the transfer.</p>
    </div>);
  }

  renderForm () {
    const selectedMeta = metaDataKeys[this.state.metaKeyIndex];

    return (
      <div>
        <SelectField
          floatingLabelText='Choose the meta-data to add'
          fullWidth
          value={ this.state.metaKeyIndex }
          onChange={ this.onMetaKeyChange }
        >

          { this.renderMetaKeyItems() }

        </SelectField>

        <InputText
          key={ selectedMeta.value }
          floatingLabelText={ `${selectedMeta.label} value` }
          hintText={ `The value of the ${selectedMeta.label.toLowerCase()} (${selectedMeta.validation === ADDRESS_TYPE ? 'Address' : 'Url Hint'})` }

          validationType={ selectedMeta.validation }
          onChange={ this.onChange }
        />
      </div>
    );
  }

  renderMetaKeyItems () {
    return metaDataKeys.map((key, index) => (
      <MenuItem
        value={ index }
        key={ index }
        label={ key.label } primaryText={ key.label }
      />
    ));
  }

  onShowDialog = () => {
    this.setState({ showDialog: true });
  }

  onClose = () => {
    this.setState(initState);
  }

  onAdd = () => {
    const { index } = this.props;

    const keyIndex = this.state.metaKeyIndex;
    const key = metaDataKeys[keyIndex].value;

    this.props.handleAddMeta(
      index,
      key,
      this.state.form.value
    );

    this.setState({ complete: true });
  }

  onChange = (valid, value) => {
    this.setState({
      form: {
        valid, value
      }
    });
  }

  onMetaKeyChange = (event, metaKeyIndex) => {
    this.setState({ metaKeyIndex, form: {
      ...[this.state.form],
      valid: false,
      value: ''
    } });
  }
}
