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
import { FormattedMessage } from 'react-intl';
import AutoComplete from 'material-ui/AutoComplete';

import styles from './EditableValue.css';
import valueStyles from '../Value/Value.css';

export default class EditableValue extends Component {
  state = {
    value: this.props.value,
    inEditMode: false
  }

  componentWillReceiveProps (newProps) {
    if (newProps.value === this.state.value || this.state.inEditMode) {
      return;
    }
    this.setState({
      value: newProps.value
    });
  }

  onChange = value => {
    this.setState({
      value: value
    });
  }

  onOpenEdit = evt => {
    this.setState({
      inEditMode: true
    });

    if (!this._input) {
      return;
    }
    this._input.focus();
  }

  onCancel = evt => {
    this.setState({
      inEditMode: false,
      value: this.props.value
    });
  }

  onSubmit = () => {
    this.setState({
      inEditMode: false
    });
    this.props.onSubmit(this.state.value, false);
  }

  onResetToDefault = () => {
    this.props.onSubmit(this.props.defaultValue, true);
  }

  render () {
    return (
      <form
        className={ `${valueStyles.valueContainer} ${styles.container}` }
        onSubmit={ this.onSubmit }
        { ...this._testInherit() }
      >
        { this.renderResetButton() }
        <div className={ this.state.inEditMode ? styles.iconsVisible : styles.icons }>
          { this.props.children }
          { this.renderButtons() }
        </div>
        { this.renderInput() }
      </form>
    );
  }

  renderInput () {
    const { inEditMode, value } = this.state;

    const setInput = el => { this._input = el; };
    const onChange = evt => this.onChange(evt.target.value);

    if (!inEditMode || !this.props.autocomplete) {
      return (
        <input
          className={ inEditMode ? styles.input : valueStyles.value }
          type='text'
          value={ value }
          onClick={ this.onOpenEdit }
          ref={ setInput }
          onChange={ onChange }
          readOnly={ !inEditMode }
        />
      );
    }

    return (
      <AutoComplete
        name='EditableValueAutoComplete' // avoid Material Ui warning
        className={ styles.autocomplete }
        fullWidth
        searchText={ value }
        dataSource={ this.props.dataSource }
        onUpdateInput={ this.onChange }
        onNewRequest={ this.onChange }
        openOnFocus
        filter={ AutoComplete.noFilter }
      />
    );
  }

  renderResetButton () {
    if (this.state.inEditMode) {
      return;
    }

    if (!this.props.defaultValue || this.state.value === this.props.defaultValue) {
      return;
    }

    return (
      <a
        key='reset'
        className={ `${styles.icon} ${styles.firstIcon}` }
        onClick={ this.onResetToDefault }
        title={
          <FormattedMessage
            id='status.editableValue.reset'
            defaultMessage='Reset to {defaultVaule}'
            values={ {
              defaultVaule: this.props.defaultValue
            } }
          />
        }
        { ...this._testInherit('reset') }
      >
        <i className='icon-anchor' />
      </a>
    );
  }

  renderButtons () {
    if (this.state.inEditMode) {
      return [
        <a
          key='submit'
          className={ styles.iconSuccess }
          onClick={ this.onSubmit }
          { ...this._testInherit('submit') }
        >
          <i className='icon-check' />
        </a>,
        <a
          key='cancel'
          className={ styles.icon }
          onClick={ this.onCancel }
          { ...this._testInherit('cancel') }
        >
          <i className='icon-close' />
        </a>
      ];
    }

    return (
      <a
        key='edit'
        className={ styles.icon }
        onClick={ this.onOpenEdit }
        title={
          <FormattedMessage
            id='status.editableValue.edit'
            defaultMessage='Edit'
          />
        }
        { ...this._testInherit('edit') }
      >
        <i className='icon-pencil' />
      </a>
    );
  }

  static propTypes = {
    onSubmit: PropTypes.func.isRequired,
    value: PropTypes.string,
    defaultValue: PropTypes.string,
    children: PropTypes.element,
    autocomplete: PropTypes.bool,
    dataSource: PropTypes.arrayOf(PropTypes.string)
  }
}
