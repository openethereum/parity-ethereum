
import React, { Component, PropTypes } from 'react';
import { isArray, isPlainObject } from 'lodash';

import styles from './Response.css';

export default class Response extends Component {

  render () {
    let { response } = this.props;
    let formatted;

    if (isArray(response)) {
      formatted = this.renderArray();
    }
    if (isPlainObject(response)) {
      formatted = this.renderObject();
    }

    return <pre className={ styles.response }>{ formatted || response }</pre>;
  }

  renderArray () {
    let { response } = this.props;
    return response.map((r, idx) => (
      <span key={ idx }>
        { idx === 0 ? '[' : ',' }
        { idx === 0 ? '' : <br /> }
        { r }
        { idx === response.length - 1 ? ']' : '' }
      </span>
    ));
  }

  renderObject () {
    let { response } = this.props;
    const arr = JSON.stringify(response, null, 1).split('\n');
    return arr.map((any, idx) => (
      <span key={ idx }>
        { any }
        { idx !== 0 && idx !== arr.length - 1 ? <br /> : '' }
      </span>
    ));
  }

  static propTypes = {
    response: PropTypes.any.isRequired
  }

}
