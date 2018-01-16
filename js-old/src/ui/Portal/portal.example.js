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

import { Button } from '~/ui';
import PlaygroundExample from '~/playground/playgroundExample';

import Portal from './portal';

export default class PortalExample extends Component {
  state = {
    open: []
  };

  render () {
    const { open } = this.state;

    return (
      <div>
        <PlaygroundExample name='Standard Portal'>
          <div>
            <button onClick={ this.handleOpen(0) }>Open</button>
            <Portal
              open={ open[0] || false }
              onClose={ this.handleClose }
            >
              <p>This is the first portal</p>
            </Portal>
          </div>
        </PlaygroundExample>

        <PlaygroundExample name='Popover Portal'>
          <div>
            <button onClick={ this.handleOpen(1) }>Open</button>
            <Portal
              isChildModal
              open={ open[1] || false }
              onClose={ this.handleClose }
            >
              <p>This is the second portal</p>
            </Portal>
          </div>
        </PlaygroundExample>

        <PlaygroundExample name='Portal in Modal'>
          <div>
            <button onClick={ this.handleOpen(2) }>Open</button>

            <Portal
              isChildModal
              open={ open[3] || false }
              onClose={ this.handleClose }
            >
              <p>This is the second portal</p>
            </Portal>
          </div>
        </PlaygroundExample>

        <PlaygroundExample name='Portal with Buttons'>
          <div>
            <button onClick={ this.handleOpen(4) }>Open</button>
            <Portal
              activeStep={ 0 }
              buttons={ [
                <Button
                  key='close'
                  label='close'
                  onClick={ this.handleClose }
                />
              ] }
              isChildModal
              open={ open[4] || false }
              onClose={ this.handleClose }
              steps={ [ 'step 1', 'step 2' ] }
              title='Portal with button'
            >
              <p>This is the fourth portal</p>
            </Portal>
          </div>
        </PlaygroundExample>
      </div>
    );
  }

  handleOpen = (index) => {
    return () => {
      const { open } = this.state;
      const nextOpen = open.slice();

      nextOpen[index] = true;
      this.setState({ open: nextOpen });
    };
  }

  handleClose = () => {
    this.setState({ open: [] });
  }
}
