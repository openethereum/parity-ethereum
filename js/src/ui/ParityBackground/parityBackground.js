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

export default class ParityBackground extends Component {
  static propTypes = {
    children: PropTypes.node,
    className: PropTypes.string,
    background: PropTypes.string.isRequired
  }

  state = {
    background: ''
  }

  componentDidMount () {
    const { background } = this.props;

    this.updateBackground(background);
  }

  componentWillReceiveProps (newProps) {
    const { background } = this.props;

    if (newProps.background === background) {
      return;
    }

    this.updateBackground(newProps.background);
  }

  render () {
    const { children, className } = this.props;
    const { background } = this.state;
    const style = { background, minHeight: '100%' };

    return (
      <div className={ className } style={ style }>
        { children }
      </div>
    );
  }

  updateBackground (seed) {
    const url = GeoPattern.generate(seed).toDataUrl();
    const background = `linear-gradient(rgba(0, 0, 0, 0.5), rgba(0, 0, 0, 0.5)), ${url}`;

    this.setState({ background });
  }
}
