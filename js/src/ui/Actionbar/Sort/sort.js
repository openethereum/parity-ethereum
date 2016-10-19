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
    order: PropTypes.string
  };

  state = {
    menuOpen: false
  }

  render () {
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
        >
        <MenuItem value='' primaryText='Default' />
        <MenuItem value='tags' primaryText='Sort by tags' />
        <MenuItem value='name' primaryText='Sort by name' />
      </IconMenu>
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
