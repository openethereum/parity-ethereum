import React, { Component } from 'react';

import Toast from '../../../Toast';
import styles from './ToastPage.css';

import ToastPageData from './ToastPage.data';

export default class ToastPage extends Component {

  render () {
    return (
      <div>
        <h1>Toast</h1>
        <div className={ styles.toastsContainer }>
          { this.renderToasts() }
        </div>
      </div>
    );
  }

  renderToasts () {
    return ToastPageData.map(t => (
      <Toast { ...t } key={ t.id } onRemoveToast={ this.onRemoveToast } />
    ));
  }

  onRemoveToast = id => {
    global.alert('remove toast ' + id);
  }

}
