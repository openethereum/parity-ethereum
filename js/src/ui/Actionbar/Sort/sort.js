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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';
import { observer } from 'mobx-react';

import Button from '~/ui/Button';
import { SortIcon } from '~/ui/Icons';
import List from '~/ui/List';
import Popup from '~/ui/Popup';

import SortStore from './sortStore';

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
      <Popup
        isOpen={ this.store.menuOpen }
        trigger={
          <Button
            icon={ <SortIcon /> }
            onClick={ this.store.handleMenuOpen }
          />
        }
      >
        <List
          items={ [
            showDefault && this.renderMenuItem('', (
              <FormattedMessage
                id='ui.actionbar.sort.typeDefault'
                defaultMessage='Default'
              />
            )),
            this.renderMenuItem('tags', (
              <FormattedMessage
                id='ui.actionbar.sort.typeTags'
                defaultMessage='Sort by tags'
              />
            )),
            this.renderMenuItem('name', (
              <FormattedMessage
                id='ui.actionbar.sort.typeName'
                defaultMessage='Sort by name'
              />
            )),
            this.renderMenuItem('eth', (
              <FormattedMessage
                id='ui.actionbar.sort.typeEth'
                defaultMessage='Sort by ETH'
              />
            ))
          ].concat(this.renderSortByMetas()) }
          onClick={ this.store.handleSortChange }
        />
      </Popup>
    );
  }

  renderSortByMetas () {
    const { metas } = this.props;

    return metas.map((meta) => {
      const label = (
        <FormattedMessage
          id='ui.actionbar.sort.sortBy'
          defaultMessage='Sort by {label}'
          values={ {
            label: meta.label
          } }
        />
      );

      return this.renderMenuItem(meta.key, label);
    });
  }

  renderMenuItem (key, label) {
    const { order } = this.props;

    return {
      isActive: order === key,
      key,
      label
    };
  }
}
