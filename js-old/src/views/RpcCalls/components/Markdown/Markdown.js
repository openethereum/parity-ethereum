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

import marked from 'marked';
import React, { Component, PropTypes } from 'react';
import styles from './Markdown.css';

export default class Marked extends Component {
  state = {}

  render () {
    let { parsed } = this.state;

    if (!parsed) {
      return null;
    }
    return <div className={ styles.container } style={ this.props.style } dangerouslySetInnerHTML={ { __html: parsed } } />;
  }

  componentWillMount () {
    this.setState({ parsed: this.parse(this.props.val) });
  }

  componentWillReceiveProps (newProps) {
    if (newProps.val === this.props.val) {
      return;
    }
    this.setState({ parsed: this.parse(newProps.val) });
  }

  parse (val) {
    try {
      val = marked(val);
    } catch (err) {
      console.error(`Marked error when parsing ${val}: ${err}`);
    }
    return val;
  }

  static propTypes = {
    val: PropTypes.any,
    style: PropTypes.object
  }
}
