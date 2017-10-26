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

import builtinsJson from '~/views/Dapps/builtin.json';

const REGISTER_URLS = {
  console: 'https://raw.githubusercontent.com/paritytech/console/3ea0dbfefded359ccdbea37bc4cf350c0aa16948/console.jpeg',
  dappreg: 'https://raw.githubusercontent.com/paritytech/dapp-assets/cdd6ac4f1e2f11619bed72a53ae71217dffe19ad/dapps/legos-64x64.png',
  githubhint: 'https://raw.githubusercontent.com/paritytech/dapp-assets/b88e983abaa1a6a6345b8d9448c15b117ddb540e/dapps/link-64x64.jpg',
  localtx: 'https://raw.githubusercontent.com/paritytech/dapp-assets/cdd6ac4f1e2f11619bed72a53ae71217dffe19ad/dapps/stack-64x64.png',
  registry: 'https://raw.githubusercontent.com/paritytech/dapp-assets/b88e983abaa1a6a6345b8d9448c15b117ddb540e/dapps/register-64x64.jpg',
  signaturereg: 'https://raw.githubusercontent.com/paritytech/dapp-assets/b88e983abaa1a6a6345b8d9448c15b117ddb540e/dapps/hex-64x64.jpg',
  tokendeploy: 'https://raw.githubusercontent.com/paritytech/dapp-assets/b88e983abaa1a6a6345b8d9448c15b117ddb540e/dapps/interlock-64x64.png',
  tokenreg: 'https://raw.githubusercontent.com/paritytech/dapp-assets/b88e983abaa1a6a6345b8d9448c15b117ddb540e/dapps/coins-64x64.jpg',
  web: 'https://raw.githubusercontent.com/paritytech/dapp-assets/ec6138115d0e1f45258969cd90b3b274e0ff2258/dapps/earth-64x64.jpg'
};

const builtins = builtinsJson
  .filter((app) => app.id)
  .map((app) => {
    app.source = {
      imageUrl: REGISTER_URLS[app.id]
    };

    return app;
  });

export {
  builtins
};
