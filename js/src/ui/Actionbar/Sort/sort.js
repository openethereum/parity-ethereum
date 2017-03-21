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
import { FormattedMessage } from 'react-intl';
import { observer } from 'mobx-react';

import IconMenu from 'material-ui/IconMenu';
import MenuItem from 'material-ui/MenuItem';

import SortIcon from 'material-ui/svg-icons/content/sort';

import Button from '../../Button';

import SortStore from './sortStore';
import styles from './sort.css';

@observer
export default class ActionbarSort extends Component {
  static propTypes = {
    id: PropTypes.string.isRequired,
    onChange: PropTypes.func.isRequired,

    order: PropTypes.string,
    showDefault: PropTypes.bool,
    metas: PropTypes.array
  };

  static defaultProps = {
    metas: [],
    showDefault: true
  }

  store = new SortStore(this.props);

  componentDidMount () {
    this.store.restoreSavedOrder();
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
            onClick={ this.store.handleMenuOpen }
          />
        }
        open={ this.store.menuOpen }
        onRequestChange={ this.store.handleMenuChange }
        onItemTouchTap={ this.store.handleSortChange }
        targetOrigin={ { horizontal: 'right', vertical: 'top' } }
        anchorOrigin={ { horizontal: 'right', vertical: 'top' } }
        touchTapCloseDelay={ 0 }
      >
        {
          showDefault
            ? this.renderMenuItem('', (
              <FormattedMessage
                id='ui.actionbar.sort.typeDefault'
                defaultMessage='Default'
              />
            ))
            : null
        }
        {
          this.renderMenuItem('tags', (
            <FormattedMessage
              id='ui.actionbar.sort.typeTags'
              defaultMessage='Sort by tags'
            />
          ))
        }
        {
          this.renderMenuItem('name', (
            <FormattedMessage
              id='ui.actionbar.sort.typeName'
              defaultMessage='Sort by name'
            />
          ))
        }
        {
          this.renderMenuItem('eth', (
            <FormattedMessage
              id='ui.actionbar.sort.typeEth'
              defaultMessage='Sort by ETH'
            />
          ))
        }

        { this.renderSortByMetas() }
      </IconMenu>
    );
  }

  renderSortByMetas () {
    const { metas } = this.props;

    return metas
      .map((meta, index) => {
        const label = (
          <FormattedMessage
            id='ui.actionbar.sort.sortBy'
            defaultMessage='Sort by {label}'
            values={ {
              label: meta.label
            } }
          />
        );

        return this.renderMenuItem(meta.key, label, index);
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
}
