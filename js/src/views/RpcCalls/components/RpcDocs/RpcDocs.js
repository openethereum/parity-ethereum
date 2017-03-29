// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import React, { Component } from 'react';
import ReactDOM from 'react-dom';
import { FormattedMessage } from 'react-intl';
import { sortBy } from 'lodash';
import List from 'material-ui/List/List';
import ListItem from 'material-ui/List/ListItem';
import AutoComplete from '../AutoComplete';

import { formatRpcMd } from '../../util/rpc-md';
import ScrollTopButton from '../ScrollTopButton';
import styles from './RpcDocs.css';
import Markdown from '../Markdown';
import rpcData from '../../data/rpc.json';
import RpcNav from '../RpcNav';

const rpcMethods = sortBy(rpcData.methods, 'name');

class RpcDocs extends Component {
  render () {
    return (
      <div className='dapp-flex-content'>
        <main className='dapp-content'>
          <div className='dapp-container'>
            <div className='row'>
              <div className='col col-6'>
                <h1>
                  <FormattedMessage
                    id='status.rpcDocs.title'
                    defaultMessage='RPC Docs'
                  />
                </h1>
              </div>
              <div className='col col-6'>
                <RpcNav />
              </div>
            </div>
          </div>
          <div style={ { clear: 'both' } } />
          <div className='dapp-container'>
            <div className='row'>
              <div className='col col-12'>
                <AutoComplete
                  floatingLabelText={
                    <FormattedMessage
                      id='status.rpcDocs.methodName'
                      defaultMessage='Method name'
                    />
                  }
                  className={ styles.autocomplete }
                  dataSource={ rpcMethods.map(m => m.name) }
                  onNewRequest={ this.handleMethodChange }
                  { ...this._test('autocomplete') }
                />
                { this.renderData() }
              </div>
            </div>
          </div>
          <ScrollTopButton />
        </main>
      </div>
    );
  }

  renderData () {
    const methods = rpcMethods.map((m, idx) => {
      const setMethod = el => { this[`_method-${m.name}`] = el; };

      return (
        <ListItem
          key={ m.name }
          disabled
          ref={ setMethod }
        >
          <h3 className={ styles.headline }>{ m.name }</h3>
          <Markdown val={ m.desc } />
          <p>
            <FormattedMessage
              id='status.rpcDocs.params'
              defaultMessage='Params {params}'
              vaules={ {
                params: !m.params.length
                  ? (
                    <FormattedMessage
                      id='status.rpcDocs.paramsNone'
                      defaultMessage=' - none'
                    />
                  )
                  : ''
              } }
            />
          </p>
          {
            m.params.map((p, idx) => {
              return (
                <Markdown
                  key={ `${m.name}-${idx}` }
                  val={ formatRpcMd(p) }
                />
              );
            })
          }
          <p className={ styles.returnsTitle }>
            <FormattedMessage
              id='status.rpcDocs.returns'
              defaultMessage='Returns - '
            />
          </p>
          <Markdown className={ styles.returnsDesc } val={ formatRpcMd(m.returns) } />
          { idx !== rpcMethods.length - 1 ? <hr /> : '' }
        </ListItem>
      );
    });

    return (
      <List>
        { methods }
      </List>
    );
  }

  handleMethodChange = name => {
    ReactDOM.findDOMNode(this[`_method-${name}`]).scrollIntoViewIfNeeded();
  }
}

export default RpcDocs;
