import React, { Component, PropTypes } from 'react';

import { Step, Stepper, StepLabel } from 'material-ui/Stepper';

import styles from './style.css';

export default class OverlaySteps extends Component {
  static propTypes = {
    current: PropTypes.number,
    steps: PropTypes.array.isRequired
  }

  render () {
    const steps = this.props.steps.map((label) => {
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
        <h3>
          { this.props.steps[this.props.current] }
        </h3>
        <Stepper
          activeStep={ this.props.current }>
          { steps }
        </Stepper>
      </div>
    );
  }
}
