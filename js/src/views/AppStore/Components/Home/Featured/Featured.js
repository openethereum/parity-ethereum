/* @flow */
import React, { Component } from 'react';

/** Components **/
import MedApp from '../../MedApp/MedApp';

/** Stylesheets **/
import styles from './Featured.css';

class Featured extends Component {
  render () {
    return (
      <div className={ styles.Featured }>

        <div className={ styles.featuredText }>
          Featured
        </div>

        <div className={ styles.dappContent }>
          <MedApp hash='0x264D14eAbB717Ea34F1540757e364727fdC75eA4' />
          <MedApp hash='0x264D14eAbB717Ea34F1540757e364727fdC75eA4' />
          <MedApp hash='0x264D14eAbB717Ea34F1540757e364727fdC75eA4' />
          <MedApp hash='0x264D14eAbB717Ea34F1540757e364727fdC75eA4' />
          <MedApp hash='0x264D14eAbB717Ea34F1540757e364727fdC75eA4' />
        </div>

      </div>
    );
  }
}

export default Featured;
