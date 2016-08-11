import React, { Component, PropTypes } from 'react';

import styles from './style.css';

export default class Form extends Component {
  static propTypes = {
    children: PropTypes.array
  }

  render () {
    // HACK: hidden inputs to disable Chrome's autocomplete
    return (
      <form
        autoComplete='off'
        className={ styles.form }>
        <div className={ styles.autofill }>
          <input type='text' name='fakeusernameremembered' />
          <input type='password' name='fakepasswordremembered' />
        </div>
        { this.props.children }
      </form>
    );
  }
}
