// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import ContentAdd from 'material-ui/svg-icons/content/add';

import brace from 'brace';
import AceEditor from 'react-ace';

import 'brace/theme/solarized_dark';
import 'brace/mode/javascript';

import { Actionbar, Button, Page } from '../../ui';

import styles from './deployContract.css';

import CompilerWorker from 'worker-loader!./compilerWorker.js';

export default class DeployContract extends Component {

  state = {
    sourceCode: ''
  };

  render () {
    const { sourceCode } = this.state;

    return (
      <div>
        { this.renderActionbar() }
        <Page>
          <div className={ styles.container }>
            <div className={ styles.editor }>
              <h2>Solidity Source Code</h2>
              <AceEditor
                mode='javascript'
                theme='solarized_dark'
                width='100%'
                onChange={ this.onEditSource }
                name='PARITY_EDITOR'
                editorProps={ { $blockScrolling: true } }
                setOptions={ {
                  showPrintMargin: false
                } }
                value={ sourceCode }
              />
            </div>
            <div className={ styles.parameters }>
              <h2>Parameters</h2>
              <Button
                label='Compile'
                onClick={ this.compile }
                primary={ false }
              />
            </div>
          </div>
        </Page>
      </div>
    );
  }

  renderActionbar () {
    const buttons = [
      <Button
        key='deployContract'
        icon={ <ContentAdd /> }
        label='deploy'
        onClick={ this.onDeployContract }
      />
    ];

    return (
      <Actionbar
        title='Write a Contract'
        buttons={ buttons }
      />
    );
  }

  onEditSource = (sourceCode) => {
    this.setState({ sourceCode });
  }

  compile = () => {
    const { sourceCode } = this.state;
    const compiler = new CompilerWorker();

    compiler.postMessage(JSON.stringify({
      action: 'compile',
      data: sourceCode
    }));

    compiler.onmessage = (event) => {
      const message = JSON.parse(event.data);
      console.log(message);
    };
  }

}
