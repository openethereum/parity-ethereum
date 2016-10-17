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
import TextField from 'material-ui/TextField';
import ActionSearch from 'material-ui/svg-icons/action/search';

import { Button } from '../../';

import styles from './search.css';

export default class ActionbarSearch extends Component {
  static propTypes = {
    onChange: PropTypes.func.isRequired
  };

  state = {
    showSearch: false,
    searchValue: '',
    stateChanging: false
  }

  render () {
    const { onChange } = this.props;
    const { showSearch } = this.state;

    const onSearchClick = () => {
      this.handleOpenSearch(!this.state.showSearch);
    };

    const onSearchBlur = () => {
      if (this.state.searchValue.length === 0) {
        this.handleOpenSearch(false);
      }
    };

    const onSearchChange = (event) => {
      const searchValue = event.target.value;

      this.setState({ searchValue });
      onChange(searchValue);
    };

    const searchInputStyle = {};

    if (!showSearch) {
      searchInputStyle.width = 0;
    }

    return (<div key='searchAccount'>
      <TextField
        className={ styles.searchInput }
        style={ searchInputStyle }
        hintText='Enter search input...'
        hintStyle={ {
          maxWidth: '100%',
          overflow: 'hidden'
        } }
        ref='searchInput'
        onBlur={ onSearchBlur }
        onChange={ onSearchChange } />

      <Button
        className={ styles.searchButton }
        icon={ <ActionSearch /> }
        label=''
        onClick={ onSearchClick } />
    </div>);
  }

  handleOpenSearch = (showSearch) => {
    if (this.state.stateChanging) return false;

    this.setState({
      showSearch: showSearch,
      stateChanging: true
    });

    if (showSearch) {
      this.refs.searchInput.focus();
    }

    window.setTimeout(() => {
      this.setState({ stateChanging: false });
    }, 450);
  }
}
