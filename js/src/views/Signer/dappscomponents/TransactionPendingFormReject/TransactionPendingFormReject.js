import React, { Component, PropTypes } from 'react';

import RaisedButton from 'material-ui/RaisedButton';

import { REJECT_COUNTER_TIME } from '../constants/constants';
import styles from './TransactionPendingFormReject.css';

export default class TransactionPendingFormReject extends Component {

  static propTypes = {
    onReject: PropTypes.func.isRequired,
    className: PropTypes.string,
    rejectCounterTime: PropTypes.number
  };

  static defaultProps = {
    rejectCounterTime: REJECT_COUNTER_TIME
  };

  state = {
    rejectCounter: this.props.rejectCounterTime
  }

  componentWillMount () {
    this.onInitCounter();
  }

  componentWillUnmount () {
    this.onResetCounter();
  }

  render () {
    const { rejectCounter } = this.state;
    const { onReject } = this.props;

    return (
      <div>
        <div className={ styles.rejectText }>
          Are you sure you want to reject transaction? <br />
          <strong>This cannot be undone</strong>
        </div>
        <RaisedButton
          onClick={ onReject }
          className={ styles.rejectButton }
          disabled={ rejectCounter > 0 }
          fullWidth
          >
          Reject Transaction { this.renderCounter() }
        </RaisedButton>
      </div>
    );
  }

  renderCounter () {
    const { rejectCounter } = this.state;
    if (!rejectCounter) {
      return;
    }
    return (
      <span>{ `(${rejectCounter})` }</span>
    );
  }

  onInitCounter () {
    this.rejectInterval = setInterval(() => {
      let { rejectCounter } = this.state;
      if (rejectCounter === 0) {
        return clearInterval(this.rejectInterval);
      }
      this.setState({ rejectCounter: rejectCounter - 1 });
    }, 1000);
  }

  onResetCounter () {
    clearInterval(this.rejectInterval);
    this.setState({
      rejectCounter: this.props.rejectCounterTime
    });
  }
}
