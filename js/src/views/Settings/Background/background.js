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

import { Container, ContainerTitle, ParityBackground } from '../../../ui';

import layout from '../layout.css';
import styles from './background.css';

let counter = 0;

export default class Background extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    muiTheme: PropTypes.object.isRequired
  }

  state = {
    seeds: []
  }

  componentDidMount () {
    const { muiTheme } = this.context;

    this.setState({
      seeds: [muiTheme.backgroundSeed]
    }, () => this.addSeeds(15));
  }

  render () {
    return (
      <Container>
        <ContainerTitle title='background' />
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <div>Manage your unique, fingerprinted application background.</div>
            <div>The bckground is derived from the secure token shared between the fron-end and Parity, and it unique accross connections. Apart from allowing you to customize the look of your UI, it also allow you to uniquely identify that you are indeed connected to a know endpoint.</div>
          </div>
          <div className={ layout.details }>
            <div className={ styles.bgcontainer }>
              { this.renderBackgrounds() }
            </div>
          </div>
        </div>
      </Container>
    );
  }

  renderBackgrounds () {
    const { seeds } = this.state;

    return seeds.map((seed) => {
      return (
        <div className={ styles.bg }>
          <ParityBackground
            className={ styles.seed }
            key={ seed }
            seed={ seed }
            onTouchTap={ this.onSelect(seed) } />
        </div>
      );
    });
  }

  onSelect = (seed) => {
    const { muiTheme } = this.context;

    return (event) => {
      muiTheme.setBackgroundSeed(seed);
    };
  }

  addSeeds (count) {
    const { seeds } = this.state;
    const newSeeds = [];

    for (let index = 0; index < count; index++) {
      newSeeds.push(this.generateSeed());
    }

    this.setState({
      seeds: seeds.concat(newSeeds)
    });
  }

  generateSeed () {
    const { api, muiTheme } = this.context;

    return api.util.sha3(`${muiTheme.backgroundSeed}${Math.random()}${counter++}`);
  }
}
