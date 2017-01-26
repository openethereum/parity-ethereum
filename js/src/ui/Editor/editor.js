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

import React, { PropTypes, Component } from 'react';

import 'brace';
import AceEditor from 'react-ace';
import { noop } from 'lodash';

import 'brace/theme/solarized_dark';
import 'brace/mode/json';
import './mode-solidity';

export default class Editor extends Component {
  static propTypes = {
    className: PropTypes.string,
    value: PropTypes.string,
    mode: PropTypes.string,
    maxLines: PropTypes.number,
    annotations: PropTypes.array,
    onExecute: PropTypes.func,
    onChange: PropTypes.func,
    readOnly: PropTypes.bool
  };

  static defaultProps = {
    className: '',
    value: '',
    mode: 'javascript',
    annotations: [],
    onExecute: noop,
    onChange: noop,
    readOnly: false
  };

  componentWillMount () {
    this.name = `PARITY_EDITOR_${Math.round(Math.random() * 99999)}`;
  }

  render () {
    const { className, annotations, value, readOnly, mode, maxLines } = this.props;
    const commands = [
      {
        name: 'execut',
        bindKey: { win: 'Ctrl-Enter', mac: 'Command-Enter' },
        exec: this.handleExecute
      }
    ];

    const max = (maxLines !== undefined)
      ? maxLines
      : (readOnly ? value.split('\n').length + 1 : null);

    return (
      <AceEditor
        mode={ mode }
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
          fontSize: '0.9em'
        } }
        maxLines={ max }
        enableBasicAutocompletion={ !readOnly }
        showPrintMargin={ false }
        annotations={ annotations }
        value={ value }
        commands={ commands }
        readOnly={ readOnly }
        className={ className }
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
