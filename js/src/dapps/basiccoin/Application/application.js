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

import React, { Component, PropTypes } from 'react';

import Header from './Header';
import Loading from './Loading';
import PAGES from './pages';

import styles from './application.css';

export default class Application extends Component {
  static propTypes = {
    children: PropTypes.node.isRequired
  }

  state = {
    loading: true
  }

  componentDidMount () {
    this.setState({ loading: false });
  }

  render () {
    const { children } = this.props;
    const { loading } = this.state;

    if (loading) {
      return (
        <Loading />
      );
    }

    const path = (window.location.hash || '').split('?')[0].split('/')[1];
    const page = PAGES.find((page) => page.path === path);
    const style = { background: page.color };

    return (
      <div className={ styles.container } style={ style }>
        <Header />
        <div className={ styles.body }>
          { children }
        </div>
      </div>
    );
  }
}
