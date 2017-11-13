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

import keycode from 'keycode';
import { isEqual } from 'lodash';
import { MenuItem, AutoComplete as MUIAutoComplete, Divider as MUIDivider } from 'material-ui';
import { PopoverAnimationVertical } from 'material-ui/Popover';
import React, { Component, PropTypes } from 'react';

import { nodeOrStringProptype } from '~/util/proptypes';

import styles from './autocomplete.css';

// Hack to prevent "Unknown prop `disableFocusRipple` on <hr> tag" error
class Divider extends Component {
  static muiName = MUIDivider.muiName;

  render () {
    return (
      <div
        style={ { margin: '0.25em 0' } }
        className={ [styles.item, styles.divider].join(' ') }
      >
        <MUIDivider style={ { height: 2 } } />
      </div>
    );
  }
}

export default class AutoComplete extends Component {
  static propTypes = {
    className: PropTypes.string,
    disabled: PropTypes.bool,
    entry: PropTypes.object,
    entries: PropTypes.oneOfType([
      PropTypes.array,
      PropTypes.object
    ]),
    error: nodeOrStringProptype(),
    filter: PropTypes.func,
    hint: nodeOrStringProptype(),
    label: nodeOrStringProptype(),
    onChange: PropTypes.func.isRequired,
    onUpdateInput: PropTypes.func,
    renderItem: PropTypes.func,
    value: PropTypes.string
  };

  state = {
    lastChangedValue: undefined,
    entry: null,
    open: false,
    dataSource: [],
    dividerBreaks: []
  };

  dividersVisibility = {};

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
    const { disabled, error, hint, label, value, className, onUpdateInput } = this.props;
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
        filter={ this.handleFilter }
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

    let currentDivider = 0;
    let firstSet = false;

    const dataSource = entriesArray.map((entry, index) => {
      // Render divider
      if (typeof entry === 'string' && entry.toLowerCase() === 'divider') {
        // Don't add divider if nothing before
        if (!firstSet) {
          return undefined;
        }

        const item = {
          text: '',
          divider: currentDivider,
          isDivider: true,
          value: (
            <Divider />
          )
        };

        currentDivider++;
        return item;
      }

      let item;

      if (renderItem && typeof renderItem === 'function') {
        item = renderItem(entry);

        // Add the item class to the entry
        const classNames = [ styles.item ].concat(item.value.props.className);

        item.value = React.cloneElement(item.value, { className: classNames.join(' ') });
      } else {
        item = {
          text: entry,
          value: (
            <MenuItem
              className={ styles.item }
              primaryText={ entry }
            />
          )
        };
      }

      if (!firstSet) {
        item.first = true;
        firstSet = true;
      }

      item.divider = currentDivider;
      item.entry = entry;

      return item;
    }).filter((item) => item !== undefined);

    return dataSource;
  }

  handleFilter = (searchText, name, item) => {
    if (item.isDivider) {
      return this.dividersVisibility[item.divider];
    }

    if (item.first) {
      this.dividersVisibility = {};
    }

    const { filter } = this.props;
    const show = filter(searchText, name, item);

    // Show the related divider
    if (show) {
      this.dividersVisibility[item.divider] = true;
    }

    return show;
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

    const { dataSource } = this.state;
    const { entry } = dataSource[idx];

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
