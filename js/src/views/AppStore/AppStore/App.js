/* @flow */
import React, { Component } from 'react';

/** Components **/
import Header       from './Header/Header';
import HeaderLights from './Header/HeaderLights';

/** Stylesheets **/
import './App.css';

type Props = {|
  children?: React.Element<*>,
|}

// type State = {|
//
// |}

// tslint:disable-next-line
class App extends Component {
  props: Props;
  // state: State = {
  //
  // };

  render() {
    return (
      <div className="App">
        <Header />
        <HeaderLights />
        {this.props.children}
      </div>
    );
  }
}

export default App;
