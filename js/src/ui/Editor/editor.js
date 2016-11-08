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

import React, { PropTypes, Component } from 'react';

import 'brace';
import AceEditor from 'react-ace';
import { noop } from 'lodash';

import 'brace/theme/solarized_dark';
import './mode-solidity';

export default class Editor extends Component {

  static propTypes = {
    value: PropTypes.string,
    annotations: PropTypes.array,
    onExecute: PropTypes.func,
    onChange: PropTypes.func,
    readOnly: PropTypes.bool
  };

  static defaultProps = {
    value: '',
    annotations: [],
    onExecute: noop,
    onChange: noop,
    readOnly: false
  };

  componentWillMount () {
    this.name = `PARITY_EDITOR_${Math.round(Math.random() * 99999)}`;
  }

  componentDidMount () {
    window.setTimeout(() => this.resize(), 1000);
  }

  resize = (editor) => {
    const editorInstance = editor || this.refs.brace.editor;
    editorInstance.resize();
  }

  render () {
    const { annotations, value, readOnly } = this.props;
    const commands = [
      {
        name: 'execut',
        bindKey: { win: 'Ctrl-Enter', mac: 'Command-Enter' },
        exec: this.handleExecute
      }
    ];

    const maxLines = readOnly ? value.split('\n').length : Infinity;

    return (
      <AceEditor
        mode='javascript'
        theme='solarized_dark'
        width='100%'
        ref='brace'
        style={ { flex: 1 } }
        onChange={ this.handleOnChange }
        name={ this.name }
        editorProps={ { $blockScrolling: Infinity } }
        setOptions={ {
          useWorker: false,
          fontFamily: 'monospace',
          fontSize: '0.9em',
          maxLines
        } }
        showPrintMargin={ false }
        annotations={ annotations }
        value={ value }
        commands={ commands }
        readOnly={ readOnly }
      />
    );
  }

  handleExecute = () => {
    this.props.onExecute();
  }

  handleOnChange = (value) => {
    this.props.onChange(value);
  }

}
