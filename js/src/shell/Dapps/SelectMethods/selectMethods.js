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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Portal } from '@parity/ui';

import MethodCheck from './MethodCheck';
import styles from './selectMethods.css';

@observer
export default class SelectMethods extends Component {
  static propTypes = {
    methodsStore: PropTypes.object.isRequired,
    visibleStore: PropTypes.object.isRequired
  };

  render () {
    const { methodsStore, visibleStore } = this.props;

    if (!methodsStore.isModalOpen) {
      return null;
    }

    return (
      <Portal
        className={ styles.modal }
        onClose={ methodsStore.closeModal }
        open
        title={
          <FormattedMessage
            id='dapps.methods.label'
            defaultMessage='allowed methods'
          />
        }
      >
        <table>
          <thead>
            <tr>
              <th>&nbsp;</th>
              {
                methodsStore.methods.map((method, methodIndex) => (
                  <th key={ methodIndex }>
                    <div>
                      <span>{ method }</span>
                    </div>
                  </th>
                ))
              }
            </tr>
          </thead>
          <tbody>
            {
              visibleStore.visibleApps.map(({ id, name }, dappIndex) => (
                <tr key={ dappIndex }>
                  <td>{ name }</td>
                  {
                    methodsStore.methods.map((method, methodIndex) => (
                      <td
                        className={ styles.check }
                        key={ `${dappIndex}_${methodIndex}` }
                      >
                        <MethodCheck
                          checked={ methodsStore.hasAppPermission(method, id) }
                          dappId={ id }
                          method={ method }
                          onToggle={ methodsStore.toggleAppPermission }
                        />
                      </td>
                    ))
                  }
                </tr>
              ))
            }
          </tbody>
        </table>
      </Portal>
    );
  }
}
