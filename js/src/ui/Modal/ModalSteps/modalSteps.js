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

import { Step, Stepper, StepLabel } from 'material-ui/Stepper';

import styles from './modalSteps.css';

export default class ModalSteps extends Component {
  static propTypes = {
    current: PropTypes.number,
    steps: PropTypes.array.isRequired,
    title: React.PropTypes.oneOfType([
      PropTypes.node, PropTypes.string
    ])
  }

  render () {
    const { current, steps, title } = this.props;
    const timeline = steps.map((label) => {
      return (
        <Step
          key={ label }>
          <StepLabel>
            { label }
          </StepLabel>
        </Step>
      );
    });

    return (
      <div
        className={ styles.title }>
        <h3>{ steps[current] }</h3>
        <div>{ title }</div>
        <Stepper
          activeStep={ current }>
          { timeline }
        </Stepper>
      </div>
    );
  }
}
