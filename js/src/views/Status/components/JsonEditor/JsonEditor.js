
import React, { Component, PropTypes } from 'react';
import { isEqual } from 'lodash';
import formatJson from 'format-json';

import styles from './JsonEditor.css';

export default class JsonEditor extends Component {

  constructor (...args) {
    super(...args);
    let { value } = this.props;
    value = formatJson.plain(value);
    this.state = { value };
  }

  componentDidMount () {
    const mockedEvt = { target: { value: this.state.value } };
    this.onChange(mockedEvt);
  }

  componentWillReceiveProps (nextProps) {
    let { value } = nextProps;

    if (!isEqual(value, this.props.value)) {
      value = formatJson.plain(value);
      this.setState({ value });
    }
  }

  render () {
    let errorClass = this.state.error ? styles.error : '';

    return (
      <div className='row'>
        <textarea
          onChange={ this.onChange }
          className={ `${styles.editor} ${errorClass}` }
          value={ this.state.value }
          />
          { this.renderError() }
      </div>
    );
  }

  renderError () {
    const { error } = this.state;
    if (!error) {
      return;
    }

    return (
      <div className={ styles.errorMsg }>{ error }</div>
    );
  }

  onChange = evt => {
    const { value } = evt.target;
    let parsed;
    let error;

    try {
      parsed = JSON.parse(value);
      error = null;
    } catch (err) {
      parsed = null;
      error = 'invalid json';
    }

    this.setState({
      value,
      error
    });

    this.props.onChange(parsed, error);
  }

  static propTypes = {
    onChange: PropTypes.func.isRequired,
    value: PropTypes.object
  }

}
