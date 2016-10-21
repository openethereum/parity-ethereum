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
import { Chip } from 'material-ui';
import { blue300 } from 'material-ui/styles/colors';
// import ChipInput from 'material-ui-chip-input';
import ChipInput from 'material-ui-chip-input/src/ChipInput';
import ActionSearch from 'material-ui/svg-icons/action/search';
import { uniq } from 'lodash';

import { Button } from '../../';

import styles from './search.css';

export default class ActionbarSearch extends Component {
  static propTypes = {
    onChange: PropTypes.func.isRequired,
    tokens: PropTypes.array
  };

  state = {
    showSearch: false,
    stateChanging: false,
    inputValue: '',
    timeoutIds: []
  }

  componentWillReceiveProps (nextProps) {
    const { tokens } = nextProps;

    if (tokens.length > 0 && this.props.tokens.length === 0) {
      this.handleOpenSearch(true, true);
    }
  }

  componentWillUnmount () {
    const { timeoutIds } = this.state;

    if (timeoutIds.length > 0) {
      timeoutIds.map(id => window.clearTimeout(id));
    }
  }

  render () {
    const { showSearch } = this.state;
    const { tokens } = this.props;

    const inputContainerClasses = [ styles.inputContainer ];

    if (!showSearch) {
      inputContainerClasses.push(styles.inputContainerShown);
    }

    return (
      <div
        className={ styles.searchcontainer }
        key='searchAccount'>
        <div className={ inputContainerClasses.join(' ') }>
          <ChipInput
            clearOnBlur={ false }
            className={ styles.input }
            chipRenderer={ this.chipRenderer }
            hintText='Enter search input...'
            hintStyle={ {
              transition: 'none'
            } }
            ref='searchInput'
            value={ tokens }
            onBlur={ this.handleSearchBlur }
            onRequestAdd={ this.handleTokenAdd }
            onRequestDelete={ this.handleTokenDelete }
            onUpdateInput={ this.handleInputChange } />
        </div>

        <Button
          className={ styles.searchButton }
          icon={ <ActionSearch /> }
          label=''
          onClick={ this.handleSearchClick } />
      </div>
    );
  }

  chipRenderer = (state, key) => {
    const { value, isFocused, isDisabled, handleClick, handleRequestDelete } = state;

    return (
      <Chip
        key={ key }
        className={ styles.chip }
        style={ {
          margin: '8px 8px 0 0',
          float: 'left',
          pointerEvents: isDisabled ? 'none' : undefined,
          alignItems: 'center'
        } }
        labelStyle={ {
          paddingRight: 6,
          fontSize: '0.9rem',
          lineHeight: 'initial'
        } }
        backgroundColor={ isFocused ? blue300 : 'rgba(0, 0, 0, 0.73)' }
        onTouchTap={ handleClick }
        onRequestDelete={ handleRequestDelete }
      >
        { value }
      </Chip>
    );
  }

  handleTokenAdd = (value) => {
    const { tokens } = this.props;

    const newSearchValues = uniq([].concat(tokens, value));

    this.setState({
      inputValue: ''
    });

    this.handleSearchChange(newSearchValues);
  }

  handleTokenDelete = (value) => {
    const { tokens } = this.props;

    const newSearchValues = []
      .concat(tokens)
      .filter(v => v !== value);

    this.setState({
      inputValue: ''
    });

    this.handleSearchChange(newSearchValues);
    this.refs.searchInput.focus();
  }

  handleInputChange = (value) => {
    const tokens = value.split(/[\s,;]/);

    const inputValue = (tokens.length <= 1)
      ? value
      : tokens.slice(-1)[0];

    if (tokens.length > 1) {
      const tokensToAdd = tokens.slice(0, -1);
      tokensToAdd.forEach(token => this.handleTokenAdd(token));
    }

    this.refs.searchInput.setState({
      inputValue
    });

    this.setState({ inputValue });
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
    const timeoutId = window.setTimeout(() => {
      const { inputValue } = this.state;
      const { tokens } = this.props;

      if (tokens.length === 0 && inputValue.length === 0) {
        this.handleOpenSearch(false);
      }
    }, 250);

    this.setState({
      timeoutIds: [].concat(this.state.timeoutIds, timeoutId)
    });
  }

  handleOpenSearch = (showSearch, force) => {
    if (this.state.stateChanging && !force) return false;

    this.setState({
      showSearch: showSearch,
      stateChanging: true
    });

    if (showSearch) {
      this.refs.searchInput.focus();
    } else {
      this.refs.searchInput.getInputNode().blur();
    }

    const timeoutId = window.setTimeout(() => {
      this.setState({ stateChanging: false });
    }, 450);

    this.setState({
      timeoutIds: [].concat(this.state.timeoutIds, timeoutId)
    });
  }
}
