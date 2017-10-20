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

import { chunkArray } from '~/util/array';
import { arrayOrObjectProptype, nodeOrStringProptype } from '~/util/proptypes';

import styles from './sectionList.css';

// TODO: We probably want this to be passed via props - additional work required in that case to
// support the styling for both the hover and no-hover CSS for the pre/post sizes. Future work only
// if/when required.
const ITEMS_PER_ROW = 3;

export default class SectionList extends Component {
  static propTypes = {
    className: PropTypes.string,
    items: arrayOrObjectProptype().isRequired,
    renderItem: PropTypes.func.isRequired,
    noStretch: PropTypes.bool,
    overlay: nodeOrStringProptype()
  };

  static defaultProps = {
    noStretch: false
  };

  render () {
    const { className, items } = this.props;

    if (!items || !items.length) {
      return null;
    }

    const rendered = items
      .map(this.renderItem)
      .filter((item) => item);

    if (!rendered.length) {
      return null;
    }

    return (
      <section className={ [styles.section, className].join(' ') }>
        { this.renderOverlay() }
        { chunkArray(rendered, ITEMS_PER_ROW).map(this.renderRow) }
      </section>
    );
  }

  renderOverlay () {
    const { overlay } = this.props;

    if (!overlay) {
      return null;
    }

    return (
      <div className={ styles.overlay }>
        { overlay }
      </div>
    );
  }

  renderRow = (row, index) => {
    return (
      <div
        className={ styles.row }
        key={ `row_${index}` }
      >
        { row }
      </div>
    );
  }

  renderItem = (item, index) => {
    const { noStretch, renderItem } = this.props;
    const itemRendered = renderItem(item, index);

    if (!itemRendered) {
      return null;
    }

    return (
      <div
        className={ [
          styles.item,
          noStretch
            ? styles.stretchOff
            : styles.stretchOn
        ].join(' ') }
        key={ `item_${index}` }
      >
        { itemRendered }
      </div>
    );
  }
}
