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

import brokenLinkImg from '../broken-link.svg';
import { parityNode } from '../../../environment';
import Hash from './hash';

import styles from './image.css';

export default class Image extends Component {
  static propTypes = {
    address: PropTypes.string
  };

  state = {
    error: false
  };

  componentWillReceiveProps (nextProps) {
    if (nextProps.address !== this.props.address) {
      this.setState({ error: false });
    }
  }

  render () {
    const { address } = this.props;

    if (!address || /^(0x)?0*$/.test(address)) {
      return (
        <code>
          No image
        </code>
      );
    }

    if (this.state.error) {
      return this.renderError();
    }

    return (
      <img
        alt={ address }
        className={ styles.image }
        onError={ this.handleError }
        src={ `${parityNode}/api/content/${address.replace(/^0x/, '')}` }
      />
    );
  }

  renderError () {
    const { address } = this.props;

    return (
      <div className={ styles.error }>
        <img
          className={ styles.image }
          src={ brokenLinkImg }
        />
        Could not load image at
        <Hash
          hash={ address }
        />
      </div>
    );
  }

  handleError = () => {
    this.setState({ error: true });
  };
}
