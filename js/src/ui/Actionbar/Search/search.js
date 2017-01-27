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
import ActionSearch from 'material-ui/svg-icons/action/search';

import Button from '../../Button';
import InputChip from '../../Form/InputChip';

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
        key='searchAccount'
      >
        <div className={ inputContainerClasses.join(' ') }>
          <InputChip
            addOnBlur
            className={ styles.input }
            hint='Enter search input...'
            ref='inputChip'
            tokens={ tokens }

            onBlur={ this.handleSearchBlur }
            onInputChange={ this.handleInputChange }
            onTokensChange={ this.handleTokensChange }
          />
        </div>

        <Button
          className={ styles.searchButton }
          icon={ <ActionSearch /> }
          label=''
          onClick={ this.handleSearchClick }
        />
      </div>
    );
  }

  handleTokensChange = (tokens) => {
    this.handleSearchChange(tokens);
  }

  handleInputChange = (inputValue) => {
    this.setState({ inputValue }, () => {
      this.handleSearchChange();
    });
  }

  handleSearchChange = (searchTokens) => {
    const { onChange, tokens } = this.props;
    const { inputValue } = this.state;

    const newSearchTokens = []
      .concat(searchTokens || tokens);

    const newSearchValues = []
      .concat(searchTokens || tokens, inputValue);

    onChange(newSearchTokens, newSearchValues);
  }

  handleSearchClick = () => {
    const { showSearch } = this.state;

    if (!showSearch) {
      this.refs.inputChip.focus();
    }

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
    if (this.state.stateChanging && !force) {
      return false;
    }

    this.setState({
      showSearch: showSearch,
      stateChanging: true
    });

    const timeoutId = window.setTimeout(() => {
      this.setState({ stateChanging: false });
    }, 450);

    this.setState({
      timeoutIds: [].concat(this.state.timeoutIds, timeoutId)
    });
  }
}
