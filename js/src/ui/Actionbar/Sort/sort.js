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
import IconMenu from 'material-ui/IconMenu';
import MenuItem from 'material-ui/MenuItem';

import SortIcon from 'material-ui/svg-icons/content/sort';

import { Button } from '../../';

import styles from './sort.css';

export default class ActionbarSort extends Component {
  static propTypes = {
    onChange: PropTypes.func.isRequired,
    order: PropTypes.string,
    showDefault: PropTypes.bool,
    metas: PropTypes.array
  };

  static defaultProps = {
    metas: [],
    showDefault: true
  }

  state = {
    menuOpen: false
  }

  render () {
    const { showDefault } = this.props;

    return (
      <IconMenu
        iconButtonElement={
          <Button
            className={ styles.sortButton }
            label=''
            icon={ <SortIcon /> }
            onClick={ this.handleMenuOpen }
            />
        }
        open={ this.state.menuOpen }
        onRequestChange={ this.handleMenuChange }
        onItemTouchTap={ this.handleSortChange }
        targetOrigin={ { horizontal: 'right', vertical: 'top' } }
        anchorOrigin={ { horizontal: 'right', vertical: 'top' } }
        touchTapCloseDelay={ 0 }
      >
        {
          showDefault
          ? this.renderMenuItem('', 'Default')
          : null
        }
        { this.renderMenuItem('tags', 'Sort by tags') }
        { this.renderMenuItem('name', 'Sort by name') }
        { this.renderMenuItem('eth', 'Sort by ETH') }

        { this.renderSortByMetas() }
      </IconMenu>
    );
  }

  renderSortByMetas () {
    const { metas } = this.props;

    return metas
      .map((meta, index) => {
        return this
          .renderMenuItem(meta.key, `Sort by ${meta.label}`, index);
      });
  }

  renderMenuItem (value, label, key = null) {
    const { order } = this.props;

    const props = {};

    if (key !== null) {
      props.key = key;
    }

    const checked = order === value;

    return (
      <MenuItem
        checked={ checked }
        value={ value }
        primaryText={ label }
        innerDivStyle={ {
          paddingLeft: checked ? 50 : 16
        } }
        { ...props }
      />
    );
  }

  handleSortChange = (event, child) => {
    const order = child.props.value;
    this.props.onChange(order);
  }

  handleMenuOpen = () => {
    this.setState({ menuOpen: true });
  }

  handleMenuChange = (open) => {
    this.setState({ menuOpen: open });
  }
}
