import React, { Component } from 'react';

import { Actionbar, Page } from '../../ui';

import StatusPage from './containers/StatusPage';

import styles from './status.css';

export default class Status extends Component {
  render () {
    return (
      <div className={ styles.container }>
        <Actionbar
          title='status' />
        <Page>
          <StatusPage />
        </Page>
      </div>
    );
  }
}
