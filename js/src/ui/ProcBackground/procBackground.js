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

import GeoPattern from 'geopattern';
import React, { Component, PropTypes } from 'react';

import styles from './procBackground.css';

export default class ProcBackground extends Component {
  static propTypes = {
    children: PropTypes.node,
    seed: PropTypes.string.isRequired
  }

  state = {
    background: ''
  }

  componentDidMount () {
    const { seed } = this.props;

    this.updateBackground(seed);
  }

  componentWillReceiveProps (newProps) {
    const { seed } = this.props;

    if (newProps.seed === seed) {
      return;
    }

    this.updateBackground(newProps.seed);
  }

  render () {
    const { children } = this.props;
    const { background } = this.state;
    const style = { background };

    return (
      <div className={ styles.background } style={ style }>
        <div className={ styles.content }>
          { children }
        </div>
      </div>
    );
  }

  updateBackground (seed) {
    const url = GeoPattern.generate(seed).toDataUrl();
    const background = `linear-gradient(rgba(0, 0, 0, 0.5), rgba(0, 0, 0, 0.5)), ${url}`;

    this.setState({ background });
  }
}
