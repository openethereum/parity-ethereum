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

import { Container, ContainerTitle } from '../../../ui';

import layout from '../layout.css';
import styles from './proxy.css';

export default class Proxy extends Component {
  render () {
    const proxyurl = 'http://127.0.0.1:8080/proxy/proxy.pac';

    return (
      <Container>
        <ContainerTitle title='Proxy' />
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <div>The proxy setup allows you to access Parity and all associated decentralized applications via memororable addresses.</div>
          </div>
          <div className={ layout.details }>
            <div className={ styles.details }>
              <div>Instead of accessing Parity via the IP address and port, you will be able to access it via the .parity subdomain, by visiting <span className={ layout.console }>http://home.parity/</span>. To setup subdomain-based routing, you need to add the relevant proxy entries to your browser,</div>
              <div className={ layout.center }><a href={ proxyurl } target='_blank'>{ proxyurl }</a></div>
              <div>To learn how to configure the proxy, instructions are provided for <a href='https://blogs.msdn.microsoft.com/ieinternals/2013/10/11/understanding-web-proxy-configuration/' target='_blank'>Windows</a>, <a href='https://support.apple.com/kb/PH18553?locale=en_US' target='_blank'>Max OS X</a> or <a href='http://xmodulo.com/how-to-set-up-proxy-auto-config-on-ubuntu-desktop.html' target='_blank'>Ubuntu</a>.</div>
            </div>
          </div>
        </div>
      </Container>
    );
  }
}
