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
import ContentSave from 'material-ui/svg-icons/content/save';
// import ChipInput from 'material-ui-chip-input';
import ChipInput from 'material-ui-chip-input/src/ChipInput';

import { Button, Form, Input, Modal } from '../../ui';
import { validateName } from '../../util/validation';

export default class EditMeta extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  }

  static propTypes = {
    keys: PropTypes.array.isRequired,
    account: PropTypes.object.isRequired,
    onClose: PropTypes.func.isRequired
  }

  state = {
    meta: Object.assign({}, this.props.account.meta),
    metaErrors: {},
    name: this.props.account.name,
    nameError: null
  }

  render () {
    const { name, nameError } = this.state;

    return (
      <Modal
        visible
        actions={ this.renderActions() }
        title='edit metadata'>
        <Form>
          <Input
            label='name'
            value={ name }
            error={ nameError }
            onSubmit={ this.onNameChange } />
          { this.renderMetaFields() }
          { this.renderTags() }
        </Form>
      </Modal>
    );
  }

  renderActions () {
    const { nameError } = this.state;

    return [
      <Button
        label='Cancel'
        icon={ <ContentClear /> }
        onClick={ this.props.onClose } />,
      <Button
        disabled={ !!nameError }
        label='Save'
        icon={ <ContentSave /> }
        onClick={ this.onSave } />
    ];
  }

  renderMetaFields () {
    const { keys } = this.props;
    const { meta } = this.state;

    return keys.map((key) => {
      const onSubmit = (value) => this.onMetaChange(key, value);
      const label = `(optional) ${key}`;
      const hint = `the optional ${key} metadata`;

      return (
        <Input
          key={ key }
          label={ label }
          hint={ hint }
          value={ meta[key] || '' }
          onSubmit={ onSubmit } />
      );
    });
  }

  renderTags () {
    const { meta } = this.state;
    const { tags } = meta || [];

    return (
      <ChipInput
        ref='tagsInput'
        value={ tags }
        onRequestAdd={ this.onAddTag }
        onRequestDelete={ this.onDeleteTag }
        floatingLabelText='(optional) tags'
        hintText='press <Enter> to add a tag'
        onUpdateInput={ this.onTagsInputChange }
        floatingLabelFixed
        fullWidth
      />
    );
  }

  onAddTag = (tag) => {
    const { meta } = this.state;
    const { tags } = meta || [];

    this.onMetaChange('tags', [].concat(tags, tag));
  }

  onDeleteTag = (tag) => {
    const { meta } = this.state;
    const { tags } = meta || [];

    const newTags = tags
      .filter(t => t !== tag);

    this.onMetaChange('tags', newTags);
  }

  onTagsInputChange = (value) => {
    const { meta } = this.state;
    const { tags = [] } = meta;

    const tokens = value.split(/[\s,;]+/);

    const newTokens = tokens
      .slice(0, -1)
      .filter(t => t.length > 0);

    const inputValue = tokens.slice(-1)[0].trim();

    this.onMetaChange('tags', [].concat(tags, newTokens));
    this.refs.tagsInput.setState({ inputValue });
  }

  onNameChange = (name) => {
    this.setState(validateName(name));
  }

  onMetaChange = (key, value) => {
    const { meta } = this.state;

    this.setState({
      meta: Object.assign(meta, { [key]: value })
    });
  }

  onSave = () => {
    const { api, store } = this.context;
    const { account } = this.props;
    const { name, nameError, meta } = this.state;

    if (nameError) {
      return;
    }

    Promise
      .all([
        api.personal.setAccountName(account.address, name),
        api.personal.setAccountMeta(account.address, Object.assign({}, account.meta, meta))
      ])
      .then(() => this.props.onClose())
      .catch((error) => {
        console.error('onSave', error);
        store.dispatch({ type: 'newError', error });
      });
  }
}
