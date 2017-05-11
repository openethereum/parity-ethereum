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
import { Link } from 'react-router';

import { nodeOrStringProptype } from '@parity/shared/util/proptypes';

import DappLink from '../DappLink';
import Title from './Title';

import styles from './container.css';

export default class Container extends Component {
  static propTypes = {
    children: PropTypes.node,
    className: PropTypes.string,
    compact: PropTypes.bool,
    dappLink: PropTypes.bool,
    hover: PropTypes.node,
    light: PropTypes.bool,
    link: PropTypes.string,
    onClick: PropTypes.func,
    onFocus: PropTypes.func,
    style: PropTypes.object,
    tabIndex: PropTypes.number,
    title: nodeOrStringProptype()
  }

  render () {
    const { children, className, compact, light, link, onClick, onFocus, style, tabIndex } = this.props;
    const props = {};

    if (Number.isInteger(tabIndex)) {
      props.tabIndex = tabIndex;
    }

    const card = (
      <div
        className={
          compact
            ? styles.compact
            : styles.padded
        }
        onClick={ onClick }
        onFocus={ onFocus }
      >
        { this.renderTitle() }
        { children }
      </div>
    );

    return (
      <div
        className={
          [
            styles.container,
            light
              ? styles.light
              : '',
            className
          ].join(' ')
        }
        style={ style }
        { ...props }
      >
        {
          link
            ? this.renderLink(link, card)
            : (
              <div>
                { card }
                { this.renderHover() }
              </div>
            )
        }
      </div>
    );
  }

  renderLink (link, card) {
    const { dappLink } = this.props;

    if (dappLink) {
      return (
        <DappLink
          className={ styles.link }
          to={ link }
        >
          { card }
          { this.renderHover() }
        </DappLink>
      );
    }

    return (
      <Link
        className={ styles.link }
        to={ link }
      >
        { card }
        { this.renderHover() }
      </Link>
    );
  }

  renderHover () {
    const { hover } = this.props;

    if (!hover) {
      return null;
    }

    return (
      <div className={ styles.hoverOverlay }>
        { hover }
      </div>
    );
  }

  renderTitle () {
    const { title } = this.props;

    if (!title) {
      return null;
    }

    return (
      <Title title={ title } />
    );
  }
}
