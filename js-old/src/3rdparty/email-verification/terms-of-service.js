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

import React from 'react';

export default (
  <ul>
    <li>This privacy notice relates to your use of the Parity email verification service. We take your privacy seriously and deal in an honest, direct and transparent way when it comes to your data.</li>
    <li>We collect your email address when you use this service. This is temporarily kept in memory, and then encrypted and stored in our EU servers. We only retain the cryptographic hash of the email address to prevent duplicated accounts. The cryptographic hash of your email address is also stored on the blockchain which is public by design. You consent to this use.</li>
    <li>You pay a fee for the cost of this service using the account you want to verify.</li>
    <li>Your email address is transmitted to a third party EU email verification service mailjet for the sole purpose of the email verification. You consent to this use. Mailjet's privacy policy is here: <a href='https://www.mailjet.com/privacy-policy'>https://www.mailjet.com/privacy-policy</a>.</li>
    <li><i>Parity Technology Limited</i> is registered in England and Wales under company number <code>09760015</code> and complies with the Data Protection Act 1998 (UK). You may contact us via email at <a href={ 'mailto:admin@parity.io' }>admin@parity.io</a>. Our general privacy policy can be found here: <a href={ 'https://parity.io/legal.html' }>https://parity.io/legal.html</a>.</li>
  </ul>
);
