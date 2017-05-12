/* @flow */
import React, { Component } from 'react';

/** Components **/
import Header from './Header/Header';
import HeaderLights from './Header/HeaderLights';

/** Stylesheets **/
import styles from './App.css';

type Props = {|
  children?: React.Element<*>,
|}

// type State = {|
//
// |}

class App extends Component {
  props: Props;
  // state: State = {
  //
  // };

  render () {
    return (
      <div className={ styles.App }>
        <Header />
        <HeaderLights />
        {this.props.children}
      </div>
    );
  }
}

export default App;
