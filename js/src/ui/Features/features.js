// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { Checkbox } from 'material-ui';
import { List, ListItem } from 'material-ui/List';
import React, { Component, PropTypes } from 'react';

import defaults, { MODES } from './defaults';
import Store from './store';
import styles from './features.css';

const isProductionMode = process.env.NODE_ENV === 'production';

export default class Features extends Component {
  static propTypes = {
    visible: PropTypes.bool.isRequired
  };

  static defaultProps = {
    visible: !isProductionMode
  };

  store = new Store();

  render () {
    if (!this.props.visible) {
      return null;
    }

    return (
      <List>
        {
          Object
            .keys(defaults)
            .filter((key) => defaults[key].mode !== MODES.PRODUCTION)
            .map(this.renderItem)
        }
      </List>
    );
  }

  renderItem = (key) => {
    const feature = defaults[key];
    const onCheck = () => feature.mode === MODES.DEVELOPMENT && this.store.toggleActive(key);

    return (
      <ListItem
        key={ `feature_${key}` }
        leftCheckbox={
          <Checkbox
            checked={ this.store.active[key] }
            disabled={ feature.mode === MODES.TESTING }
            onCheck={ onCheck }
          />
        }
        primaryText={ feature.name }
        secondaryText={
          <div className={ styles.description }>
            { feature.description }
          </div>
        }
      />
    );
  }
}
