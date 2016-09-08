import React, { Component, PropTypes } from 'react';

import Response from '../Response';
import styles from './Call.css';

export default class Call extends Component {

  render () {
    let { callNo, name, params, response } = this.props.call;
    params = this.formatParams(params);
    return (
      <div
        onMouseEnter={ this.setActiveCall }
        ref={ this.setElement }
        className={ styles.call }
        { ...this._test(`call-${callNo}`) }
        >
        <span className={ styles.callNo } { ...this._test('callNo') }>#{ callNo }</span>
        <pre { ...this._test('name') }>{ name }({ params })</pre>
        <Response response={ response } />
      </div>
    );
  }

  setElement = el => {
    this.element = el;
  }

  setActiveCall = () => {
    this.props.setActiveCall(this.props.call, this.element);
  }

  formatParams (params) {
    return params.reduce((str, p) => {
      if (str !== '') {
        str += ', ';
      }
      if (p === undefined) {
        return str;
      }
      if (typeof p === 'object' || typeof p === 'string') {
        p = JSON.stringify(p);
      }
      return str + p;
    }, '');
  }

  static propTypes = {
    call: PropTypes.object.isRequired,
    setActiveCall: PropTypes.func.isRequired
  }

}
