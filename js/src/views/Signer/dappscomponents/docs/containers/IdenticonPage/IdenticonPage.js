import React, { Component } from 'react';

import Identicon from '../../../Identicon';
import styles from './IdenticonPage.css';

import identiconPageData from './IdenticonPage.data';

export default class IdenticonPage extends Component {

  render () {
    return (
      <div>
        <h1>Identicon</h1>
        { this.renderIdenticons() }
      </div>
    );
  }

  renderIdenticons () {
    return identiconPageData.map(idc => {
      return (
        <div className={ styles.idcContainer } key={ idc.address }>
          <Identicon { ...idc } className={ styles.idc } />
          { this.renderIdenticonInfo(idc) }
        </div>
      );
    });
  }

  renderIdenticonInfo (idc) {
    return (
      <div className={ styles.idcInfo }>
        <div>Chain: { idc.chain }</div>
        <div>Address: { idc.address }</div>
      </div>
    );
  }

}
