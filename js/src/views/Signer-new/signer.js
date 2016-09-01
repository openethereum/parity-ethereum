import React, { Component } from 'react';

import Container from '../../ui/Container';

import styles from './style.css';

export default class Settings extends Component {
  render () {
    return (
      <Container>
        <iframe
          className={ styles.iframe }
          src='http://127.0.0.1:8180/'>
        </iframe>
      </Container>
    );
  }
}
