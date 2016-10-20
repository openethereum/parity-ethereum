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
import { MenuItem, AutoComplete as MUIAutoComplete } from 'material-ui';

export default class AutoComplete extends Component {
  static propTypes = {
    onChange: PropTypes.func.isRequired,
    disabled: PropTypes.bool,
    label: PropTypes.string,
    hint: PropTypes.string,
    error: PropTypes.string,
    value: PropTypes.string,
    className: PropTypes.string,
    filter: PropTypes.func,
    renderItem: PropTypes.func,
    entries: PropTypes.oneOfType([
      PropTypes.array,
      PropTypes.object
    ])
  }

  render () {
    const { disabled, error, hint, label, value, className, filter } = this.props;

    return (
      <MUIAutoComplete
        className={ className }
        disabled={ disabled }
        floatingLabelText={ label }
        hintText={ hint }
        errorText={ error }
        onNewRequest={ this.onChange }
        searchText={ value }
        onFocus={ this.onFocus }

        filter={ filter }
        openOnFocus
        fullWidth
        floatingLabelFixed
        dataSource={ this.getDataSource() }
      />
    );
  }

  getDataSource () {
    const { renderItem, entries } = this.props;
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

  onChange = (item, idx) => {
    const { onChange, entries } = this.props;
    const entriesArray = (entries instanceof Array)
      ? entries
      : Object.values(entries);

    const entry = (idx === -1) ? null : entriesArray[idx];

    onChange(entry);
  }

  onFocus = () => {
    this.props.onChange(null);
  }

}
