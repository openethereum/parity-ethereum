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

import React, { Component, PropTypes } from 'react';

import FileSaver from 'file-saver';
import FileDownloadIcon from 'material-ui/svg-icons/file/file-download';

import { Button } from '../../';

class ActionbarExport extends Component {
  static propTypes = {
    content: PropTypes.oneOfType([
      PropTypes.string,
      PropTypes.object
    ]).isRequired,
    filename: PropTypes.string.isRequired,
    className: PropTypes.string
  }

  render () {
    const { className } = this.props;

    return (
      <Button
        className={ className }
        icon={ <FileDownloadIcon /> }
        label='export'
        onClick={ this.handleExport }
      />
    );
  }

  onDownloadBackup = (filetype) => {
    const { filename, content } = this.props;

    const text = this.contentAsString(content, filetype);
    const extension = this.getExtension(filetype);

    const blob = new Blob([ text ], { type: 'text/plain;charset=utf-8' });
    FileSaver.saveAs(blob, `${filename}.${extension}`);
  }

  getExtension = (filetype) => {
    switch (filetype) {
      case 'json':
        return filetype;
      default:
        return 'txt';
    }
  }

  contentAsString = (data, filetype) => {
    if (typeof data === 'string') {
      return data;
    }

    switch (filetype) {
      case 'json':
        return JSON.stringify(data, null, 4);
      default:
        return data.toString();
    }
  }

  handleExport = () => {
    this.onDownloadBackup('json');
  }
}

export default ActionbarExport;
