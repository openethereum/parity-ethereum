
import React, { Component } from 'react';
import { Link } from 'react-router';
import styles from './RpcNav.css';

export default class RpcNav extends Component {

  render () {
    return (
      <div className={ styles.nav }>
        <Link to={ '/rpc/calls' } activeClassName={ styles.activeNav } { ...this._test('rpc-calls-link') }>
          <i className='icon-call-out'></i>
        </Link>
        <Link to={ '/rpc/docs' } activeClassName={ styles.activeNav } { ...this._test('rpc-docs-link') }>
          <i className='icon-docs'></i>
        </Link>
      </div>
    );
  }
}
