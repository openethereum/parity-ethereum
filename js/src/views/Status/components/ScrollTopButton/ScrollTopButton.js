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
import IconButton from 'material-ui/IconButton';
import ArrowUpwardIcon from 'material-ui/svg-icons/navigation/arrow-upward';

import { scrollTo } from './util';
import styles from './ScrollTopButton.css';

const scrollTopThreshold = 600;

export default class ScrollTopButton extends Component {
  state = {}

  componentDidMount () {
    window.addEventListener('scroll', this.handleScroll);
  }

  componentWillUnmount () {
    window.removeEventListener('scroll', this.handleScroll);
  }

  _scrollToTop () {
    scrollTo(document.body, 0, 500);
  }

  render () {
    let hiddenClass = !this.state.showScrollButton ? styles.hidden : '';

    return (
      <IconButton
        className={ `${styles.scrollButton} ${hiddenClass}` }
        onTouchTap={ this._scrollToTop }
      >
        <ArrowUpwardIcon />
      </IconButton>
    );
  }

  handleScroll = event => {
    let { scrollTop } = event.srcElement.body;
    let { showScrollButton } = this.state;

    if (!showScrollButton && scrollTop > scrollTopThreshold) {
      this.setState({
        showScrollButton: true
      });
    }

    if (showScrollButton && scrollTop < scrollTopThreshold) {
      this.setState({
        showScrollButton: false
      });
    }
  }
}
