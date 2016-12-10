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
import keycode from 'keycode';
import { MenuItem, AutoComplete as MUIAutoComplete } from 'material-ui';
import { PopoverAnimationVertical } from 'material-ui/Popover';

import { isEqual } from 'lodash';

export default class AutoComplete extends Component {
  static propTypes = {
    onChange: PropTypes.func.isRequired,
    onUpdateInput: PropTypes.func,
    disabled: PropTypes.bool,
    label: PropTypes.string,
    hint: PropTypes.string,
    error: PropTypes.string,
    value: PropTypes.string,
    className: PropTypes.string,
    filter: PropTypes.func,
    renderItem: PropTypes.func,
    entry: PropTypes.object,
    entries: PropTypes.oneOfType([
      PropTypes.array,
      PropTypes.object
    ])
  }

  state = {
    lastChangedValue: undefined,
    entry: null,
    open: false,
    dataSource: []
  }

  componentWillMount () {
    const dataSource = this.getDataSource();
    this.setState({ dataSource });
  }

  componentWillReceiveProps (nextProps) {
    const prevEntries = Object.keys(this.props.entries || {}).sort();
    const nextEntries = Object.keys(nextProps.entries || {}).sort();

    if (!isEqual(prevEntries, nextEntries)) {
      const dataSource = this.getDataSource(nextProps);
      this.setState({ dataSource });
    }
  }

  render () {
    const { disabled, error, hint, label, value, className, filter, onUpdateInput } = this.props;
    const { open, dataSource } = this.state;

    return (
      <MUIAutoComplete
        className={ className }
        disabled={ disabled }
        floatingLabelText={ label }
        hintText={ hint }
        errorText={ error }
        onNewRequest={ this.onChange }
        onUpdateInput={ onUpdateInput }
        searchText={ value }
        onFocus={ this.onFocus }
        onClose={ this.onClose }
        animation={ PopoverAnimationVertical }
        filter={ filter }
        popoverProps={ { open } }
        openOnFocus
        menuCloseDelay={ 0 }
        fullWidth
        floatingLabelFixed
        dataSource={ dataSource }
        menuProps={ { maxHeight: 400 } }
        ref='muiAutocomplete'
        onKeyDown={ this.onKeyDown }
      />
    );
  }

  getDataSource (props = this.props) {
    const { renderItem, entries } = props;
    const entriesArray = (entries instanceof Array)
      ? entries
      : Object.values(entries);

    if (renderItem && typeof renderItem === 'function') {
      return entriesArray.map(entry => renderItem(entry));
    }

    return entriesArray.map(entry => ({
      text: entry,
      value: (
        <MenuItem
          primaryText={ entry }
        />
      )
    }));
  }

  onKeyDown = (event) => {
    const { muiAutocomplete } = this.refs;

    switch (keycode(event)) {
      case 'down':
        const { menu } = muiAutocomplete.refs;
        menu && menu.handleKeyDown(event);
        break;

      case 'enter':
      case 'tab':
        event.preventDefault();
        event.stopPropagation();
        event.which = 'useless';

        const e = new CustomEvent('down');
        e.which = 40;

        muiAutocomplete && muiAutocomplete.handleKeyDown(e);
        break;
    }
  }

  onChange = (item, idx) => {
    if (idx === -1) {
      return;
    }

    const { entries } = this.props;

    const entriesArray = (entries instanceof Array)
      ? entries
      : Object.values(entries);

    const entry = entriesArray[idx];

    this.handleOnChange(entry);
    this.setState({ entry, open: false });
  }

  onClose = (event) => {
    const { onUpdateInput } = this.props;

    if (!onUpdateInput) {
      const { entry } = this.state;
      this.handleOnChange(entry);
    }
  }

  onFocus = () => {
    const { entry } = this.props;

    this.setState({ entry, open: true }, () => {
      this.handleOnChange(null, true);
    });
  }

  handleOnChange = (value, empty) => {
    const { lastChangedValue } = this.state;

    if (value !== lastChangedValue) {
      this.setState({ lastChangedValue: value });
      this.props.onChange(value, empty);
    }
  }
}
