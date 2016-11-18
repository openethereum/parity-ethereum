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
import { Translate } from 'react-i18nify';

import { Container, ContainerTitle } from '../../../ui';

import layout from '../layout.css';
import styles from './proxy.css';

export default class Proxy extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  render () {
    const { dappsUrl } = this.context.api;
    const proxyurl = `${dappsUrl}/proxy/proxy.pac`;

    return (
      <Container>
        <ContainerTitle title='Proxy' />
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <div><Translate value='settings.proxy.overview_0' /></div>
          </div>
          <div className={ layout.details }>
            <div className={ styles.details }>
              <div>
                <Translate value='settings.proxy.details_0' />
                <span className={ layout.console }>http://home.parity/</span>
                <Translate value='settings.proxy.details_1' />
              </div>
              <div className={ layout.center }>
                <a href={ proxyurl } target='_blank'>{ proxyurl }</a>
              </div>
              <div>
                <Translate value='settings.proxy.details_2' />
                <a
                  href='https://blogs.msdn.microsoft.com/ieinternals/2013/10/11/understanding-web-proxy-configuration/'
                  target='_blank'>
                  <Translate value='settings.proxy.details_windows' />
                </a>
                <Translate value='settings.proxy.details_3' />
                <a
                  href='https://support.apple.com/kb/PH18553?locale=en_US'
                  target='_blank'>
                  <Translate value='settings.proxy.details_macos' />
                </a>
                <Translate value='settings.proxy.details_4' />
                <a
                  href='http://xmodulo.com/how-to-set-up-proxy-auto-config-on-ubuntu-desktop.html'
                  target='_blank'>
                  <Translate value='settings.proxy.details_ubuntu' />
                </a>
              </div>
            </div>
          </div>
        </div>
      </Container>
    );
  }
}
