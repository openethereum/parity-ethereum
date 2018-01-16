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

import React, { Component, PropTypes } from 'react';
import Dropzone from 'react-dropzone';
import { FormattedMessage } from 'react-intl';

import { nodeOrStringProptype } from '~/util/proptypes';

import styles from './fileSelect.css';

export default class FileSelect extends Component {
  static propTypes = {
    className: PropTypes.string,
    error: nodeOrStringProptype(),
    label: nodeOrStringProptype(),
    onSelect: PropTypes.func.isRequired
  }

  static defaultProps = {
    label: (
      <FormattedMessage
        id='ui.fileSelect.defaultLabel'
        defaultMessage='Drop a file here, or click to select a file to upload'
      />
    )
  }

  render () {
    const { className, error, label } = this.props;

    return (
      <Dropzone
        onDrop={ this.onDrop }
        multiple={ false }
        className={
          [
            styles.dropzone,
            error
              ? styles.error
              : '',
            className
          ].join(' ') }
      >
        <div className={ styles.label }>
          { error || label }
        </div>
      </Dropzone>
    );
  }

  onDrop = (files) => {
    const { onSelect } = this.props;
    const reader = new FileReader();
    const file = files[0];

    reader.onload = (event) => {
      onSelect(file.name, event.target.result);
    };

    reader.readAsText(file);
  }
}
