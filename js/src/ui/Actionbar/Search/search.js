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

    if (tokens.length !== this.props.tokens.length) {
      this.handleSearchChange(tokens);
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
            ref='searchInput'
            value={ tokens }
            onBlur={ this.handleSearchBlur }
            onRequestAdd={ this.handleTokenAdd }
            onRequestDelete={ this.handleTokenDelete }
            onUpdateInput={ this.handleInputChange }
            hintStyle={ {
              bottom: 16,
              left: 2,
              transition: 'none'
            } }
            inputStyle={ {
              marginBottom: 18
            } }
            textFieldStyle={ {
              height: 42
            } }
          />
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

    const newSearchTokens = uniq([].concat(tokens, value));

    this.handleSearchChange(newSearchTokens);
  }

  handleTokenDelete = (value) => {
    const { tokens } = this.props;

    const newSearchTokens = []
      .concat(tokens)
      .filter(v => v !== value);

    this.handleSearchChange(newSearchTokens);
    this.refs.searchInput.focus();
  }

  handleInputChange = (value) => {
    const splitTokens = value.split(/[\s,;]/);

    const inputValue = (splitTokens.length <= 1)
      ? value
      : splitTokens.slice(-1)[0].trim();

    this.refs.searchInput.setState({ inputValue });
    this.setState({ inputValue }, () => {
      if (splitTokens.length > 1) {
        const tokensToAdd = splitTokens.slice(0, -1);
        tokensToAdd.forEach(token => this.handleTokenAdd(token));
      } else {
        this.handleSearchChange();
      }
    });
  }

  handleSearchChange = (searchTokens) => {
    const { onChange, tokens } = this.props;
    const { inputValue } = this.state;

    const newSearchTokens = []
      .concat(searchTokens || tokens)
      .filter(v => v.length > 0);

    const newSearchValues = []
      .concat(searchTokens || tokens, inputValue)
      .filter(v => v.length > 0);

    onChange(newSearchTokens, newSearchValues);
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
