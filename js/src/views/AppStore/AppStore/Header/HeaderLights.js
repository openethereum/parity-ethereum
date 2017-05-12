/* @flow */
import React, { Component } from 'react';

/** Stylesheets **/
import './Header.css';

type Props = {|

|}

// type State = {|
//
// |}

class HeaderLights extends Component {
  props: Props;
  // state: State = {
  // };

  render() {
    return (
      <div className="HeaderLights">
        <div className="light-container">
          <div className="light-bulb" />
        </div>
        <div className="light-container">
          <div className="light-bulb" />
        </div>
        <div className="light-container">
          <div className="light-bulb" />
        </div>
      </div>
    );
  }
}

export default HeaderLights;
