import React, { Component } from 'react';
import ActionFingerprint from 'material-ui/svg-icons/action/fingerprint';

import imagesEthcoreBlock from '../../images/ethcore-block.png';
import styles from './parityBar.css';

export default class ParityBar extends Component {
  render () {
    return (
      <div className={ styles.bar }>
        <div className={ styles.corner }>
          <a
            className={ styles.link }
            href='/#/apps'>
            <img src={ imagesEthcoreBlock } />
            <div>Parity</div>
          </a>
          <a
            className={ styles.link }
            href='/#/signer'>
            <ActionFingerprint />
            <div>Signer</div>
          </a>
        </div>
      </div>
    );
  }
}
