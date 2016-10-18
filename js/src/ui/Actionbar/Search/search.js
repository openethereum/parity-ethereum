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
// import ChipInput from 'material-ui-chip-input';
import ChipInput from 'material-ui-chip-input/src/ChipInput';
import ActionSearch from 'material-ui/svg-icons/action/search';
import { uniq } from 'lodash';

import { Button } from '../../';

import styles from './search.css';

export default class ActionbarSearch extends Component {
  static propTypes = {
    onChange: PropTypes.func.isRequired
  };

  state = {
    showSearch: false,
    searchValues: [],
    stateChanging: false,
    inputValue: ''
  }

  render () {
    const { showSearch, searchValues } = this.state;

    const searchInputStyle = { width: 500 };

    if (!showSearch) {
      searchInputStyle.width = 0;
    }

    return (
      <div
        className={ styles.searchcontainer }
        key='searchAccount'>
        <ChipInput
          persistInput
          style={ searchInputStyle }
          className={ styles.searchInput }
          hintText='Enter search input...'
          hintStyle={ {
            maxWidth: '100%',
            overflow: 'hidden',
            bottom: 22
          } }
          ref='searchInput'
          value={ searchValues }
          onBlur={ this.handleSearchBlur }
          onRequestAdd={ this.handleTokenAdd }
          onRequestDelete={ this.handleTokenDelete }
          onUpdateInput={ this.handleInputChange } />

        <Button
          className={ styles.searchButton }
          icon={ <ActionSearch /> }
          label=''
          onClick={ this.handleSearchClick } />
      </div>
    );
  }

  handleTokenAdd = (value) => {
    const { searchValues } = this.state;

    const newSearchValues = uniq([].concat(searchValues, value));

    this.setState({
      searchValues: newSearchValues
    });

    this.handleSearchChange(newSearchValues);
  }

  handleTokenDelete = (value) => {
    const { searchValues } = this.state;

    const newSearchValues = []
      .concat(searchValues)
      .filter(v => v !== value);

    this.setState({
      searchValues: newSearchValues
    });

    this.handleSearchChange(newSearchValues);
  }

  handleInputChange = (value) => {
    const { searchValues } = this.state;

    const newSearchValues = uniq([].concat(searchValues, value));

    this.handleSearchChange(newSearchValues);
    this.setState({ inputValue: value });
  }

  handleSearchChange = (searchValues) => {
    const { onChange } = this.props;
    const newSearchValues = searchValues.filter(v => v.length > 0);

    onChange(newSearchValues);
  }

  handleSearchClick = () => {
    const { showSearch } = this.state;

    this.handleOpenSearch(!showSearch);
  }

  handleSearchBlur = () => {
    const { searchValues, inputValue } = this.state;

    if (searchValues.length === 0 && inputValue.length === 0) {
      this.handleOpenSearch(false);
    }
  }

  handleOpenSearch = (showSearch) => {
    if (this.state.stateChanging) return false;

    this.setState({
      showSearch: showSearch,
      stateChanging: true
    });

    if (showSearch) {
      this.refs.searchInput.focus();
    } else {
      this.refs.searchInput.getInputNode().blur();
    }

    window.setTimeout(() => {
      this.setState({ stateChanging: false });
    }, 450);
  }
}
