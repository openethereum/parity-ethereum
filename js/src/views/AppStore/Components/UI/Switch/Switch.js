/* @flow */
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

/** Stylesheets **/
import styles from './Switch.css';

type Props = {|
  defaultValue: bool
|}

type State = {|
  state: bool,
  position: Object
|}

class Switch extends Component {
  props: Props;
  state: State = {
    state: false,
    position: { transform: 'translateX(-50px)' }
  }

  componentWillMount () {
    const { defaultValue } = this.props;

    if (defaultValue) {
      this.setState({
        state: true,
        position: { transform: 'translateX(0px)' }
      });
    }
  }

  switchClick = () => {
    const { state } = this.state;

    if (state) {
      this.setState({
        state: false,
        position: { transform: 'translateX(-50px)' }
      });
    } else {
      this.setState({
        state: true,
        position: { transform: 'translateX(0px)' }
      });
    }
  }

  render () {
    const { position } = this.state;

    return (
      <div className={ styles.Switch }>

        <div className={ styles.switchBodyButton } style={ position }>
          <div id={ styles.switchButton }>
            <div id={styles.switchCenterButton} />
          </div>
        </div>

        <div className={styles.switchBody} onClick={ this.switchClick }>
          <div id={styles.clicker} style={ position }>
            <div id={styles.switchLight} />
            <div id={styles.switchRightButton} />
          </div>
        </div>

      </div>
    );
  }
}

export default Switch;
