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

import { noop } from 'lodash';
import React, { Component, PropTypes } from 'react';

import { StarIcon } from '~/ui/Icons';
import SectionList from '~/ui/SectionList';
import { arrayOrObjectProptype } from '~/util/proptypes';

import styles from './selectionList.css';

export default class SelectionList extends Component {
  static propTypes = {
    isChecked: PropTypes.func,
    items: arrayOrObjectProptype().isRequired,
    noStretch: PropTypes.bool,
    onDefaultClick: PropTypes.func,
    onSelectClick: PropTypes.func,
    onSelectDoubleClick: PropTypes.func,
    renderItem: PropTypes.func.isRequired
  };

  static defaultProps = {
    onSelectDoubleClick: noop
  };

  render () {
    const { items, noStretch } = this.props;

    return (
      <SectionList
        items={ items }
        noStretch={ noStretch }
        renderItem={ this.renderItem }
      />
    );
  }

  renderItem = (item, index) => {
    const { isChecked, onDefaultClick, onSelectClick, onSelectDoubleClick, renderItem } = this.props;
    const isSelected = isChecked
      ? isChecked(item)
      : item.checked;

    const handleClick = () => {
      if (onSelectClick) {
        onSelectClick(item);
        return false;
      }
    };
    const handleDoubleClick = () => {
      onSelectDoubleClick(item);
      return false;
    };

    let defaultIcon = null;

    if (onDefaultClick) {
      const makeDefault = () => {
        onDefaultClick(item);
        return false;
      };

      defaultIcon = (
        <div className={ styles.overlay }>
          {
            isSelected && item.default
              ? <StarIcon className={ styles.icon } />
              : <StarIcon className={ styles.iconDisabled } onClick={ makeDefault } />
          }
        </div>
      );
    }

    const classes = isSelected
      ? [styles.item, styles.selected]
      : [styles.item, styles.unselected];

    if (item.default) {
      classes.push(styles.default);
    }

    return (
      <div className={ classes.join(' ') }>
        <div
          className={ styles.content }
          onClick={ handleClick }
          onDoubleClick={ handleDoubleClick }
        >
          { renderItem(item, index) }
        </div>
        { defaultIcon }
      </div>
    );
  }
}
