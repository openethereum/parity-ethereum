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
