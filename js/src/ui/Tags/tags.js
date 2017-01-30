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

import { arrayOrObjectProptype } from '~/util/proptypes';

import styles from './tags.css';

export default class Tags extends Component {
  static propTypes = {
    handleAddSearchToken: PropTypes.func,
    setRefs: PropTypes.func,
    tags: arrayOrObjectProptype()
  }

  render () {
    const { tags } = this.props;

    if (!tags || tags.length === 0) {
      return null;
    }

    return (
      <div className={ styles.tags }>
        { this.renderTags() }
      </div>
    );
  }

  renderTags () {
    const { handleAddSearchToken, setRefs, tags } = this.props;

    const tagClasses = handleAddSearchToken
      ? [ styles.tag, styles.tagClickable ]
      : [ styles.tag ];

    const setRef = setRefs
      ? (ref) => { setRefs(ref); }
      : () => {};

    return tags
      .sort()
      .map((tag, index) => {
        const onClick = handleAddSearchToken
          ? () => handleAddSearchToken(tag)
          : null;

        return (
          <div
            key={ `tag_${index}` }
            className={ tagClasses.join(' ') }
            onClick={ onClick }
            ref={ setRef }
          >
            { tag }
          </div>
        );
      });
  }
}
