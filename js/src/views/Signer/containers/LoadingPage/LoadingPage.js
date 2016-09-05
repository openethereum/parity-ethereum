import React, { Component } from 'react';
import { CircularProgress } from 'material-ui';

import { Container, ContainerTitle } from '../../../../ui';

import styles from './LoadingPage.css';

export default class LoadingPage extends Component {
  render () {
    return (
      <Container>
        <ContainerTitle
          title='Connecting to Parity' />
        <div className={ styles.main }>
          <CircularProgress size={ 2 } />
        </div>
      </Container>
    );
  }
}
