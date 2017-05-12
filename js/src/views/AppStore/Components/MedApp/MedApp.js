/* @flow */
import React, { Component } from 'react';
import { Link } from 'react-router';

/** Stylesheets **/
import './MedApp.css';

class MedApp extends Component {

  render() {
    const { hash } = this.props;

    return (
      <Link to={`/dapps/${hash}`}>
        <div className="MedApp">
          MedApp
        </div>
      </Link>
    );
  }
}

export default MedApp;
