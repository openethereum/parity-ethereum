/* @flow */
import React, { Component } from 'react';

/** Stylesheets **/
import styles from './Header.css';

// type Props = {|
//
// |}

// type State = {|
//
// |}

class HeaderLights extends Component {
  // props: Props;
  // state: State = {
  // };

  render () {
    return (
      <div className={ styles.HeaderLights }>
        <div className={ styles.lightContainer }>
          <div className={ styles.lightBulb } />
        </div>
        <div className={ styles.lightContainer }>
          <div className={ styles.lightBulb } />
        </div>
        <div className={ styles.lightContainer }>
          <div className={ styles.lightBulb } />
        </div>
      </div>
    );
  }
}

export default HeaderLights;
