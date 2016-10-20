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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import NavigationRefresh from 'material-ui/svg-icons/navigation/refresh';

import { Button, Container, ContainerTitle, ParityBackground } from '../../../ui';

import { updateBackground } from '../actions';

import layout from '../layout.css';
import styles from './background.css';

let counter = 0;

class Background extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    muiTheme: PropTypes.object.isRequired
  }

  static propTypes = {
    settings: PropTypes.object.isRequired,
    updateBackground: PropTypes.func.isRequired
  }

  state = {
    seeds: []
  }

  componentDidMount () {
    const { settings } = this.props;

    this.setState({
      seeds: [settings.backgroundSeed]
    }, () => this.addSeeds(19));
  }

  render () {
    return (
      <Container>
        <ContainerTitle title='Background Pattern' />
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <div>The background pattern you can see right now is unique to your Parity installation. It will change every time you create a new Signer token. This is so that decentralized applications cannot pretend to be trustworthy.</div>
            <div>Pick a pattern you like and memorize it. This Pattern will always be shown from now on, unless you clear your browser cache or use a new Signer token.</div>
          </div>
          <div className={ layout.details }>
            <div className={ styles.bgcontainer }>
              { this.renderBackgrounds() }
            </div>
            <Button
              icon={ <NavigationRefresh /> }
              label='generate more'
              onClick={ this.generateMore } />
          </div>
        </div>
      </Container>
    );
  }

  renderBackgrounds () {
    const { settings } = this.props;
    const { seeds } = this.state;

    return seeds.map((seed) => {
      return (
        <div className={ styles.bgflex } key={ seed }>
          <div className={ styles.bgseed }>
            <ParityBackground
              className={ settings.backgroundSeed === seed ? styles.seedactive : styles.seed }
              seed={ seed }
              onClick={ this.onSelect(seed) } />
          </div>
        </div>
      );
    });
  }

  onSelect = (seed) => {
    const { muiTheme } = this.context;
    const { updateBackground } = this.props;

    return (event) => {
      muiTheme.parity.setBackgroundSeed(seed);
      updateBackground(seed);
    };
  }

  generateMore = () => {
    this.addSeeds(20);
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

function mapStateToProps (state) {
  const { settings } = state;

  return { settings };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({ updateBackground }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Background);
