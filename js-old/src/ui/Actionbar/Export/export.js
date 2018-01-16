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
import { FormattedMessage } from 'react-intl';

import FileSaver from 'file-saver';

import Button from '../../Button';
import { FileDownloadIcon } from '../../Icons';

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
        label={
          <FormattedMessage
            id='ui.actionbar.export.button.export'
            defaultMessage='export'
          />
        }
        onClick={ this.handleExport }
      />
    );
  }

  handleExport = () => {
    const { filename, content } = this.props;
    const text = JSON.stringify(content, null, 4);
    const blob = new Blob([ text ], { type: 'application/json' });

    FileSaver.saveAs(blob, `${filename}.json`);
  }
}

export default ActionbarExport;
