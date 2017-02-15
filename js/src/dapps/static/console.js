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

/* global web3 */

if (typeof (window.parent.secureApi) === 'object') {
  window.api = window.parent.secureApi;

  if (typeof (window.Web3) === 'function') {
    Promise.all([
      window.api.parity.dappsInterface(),
      window.api.parity.dappsPort()
    ]).then(res => {
      window.web3 = new window.Web3(new window.Web3.providers.HttpProvider(`http://${res.join(':')}/rpc/`));
    });
  }
} else if (typeof (window.parity) === 'object') {
  window.api = window.parity.api;
}

if (typeof (window.api) === 'object') {
  window.api.subscribe('eth_blockNumber', function (error, blockNumber) {
    if (error) {
      console.log('error', error);
      return;
    }
    refreshWatches();
  });
}

function escapeHtml (str) {
  let div = document.createElement('div');

  div.appendChild(document.createTextNode(str));
  return div.innerHTML;
}

function getAllPropertyNames (obj) {
  let props = {};

  do {
    Object.getOwnPropertyNames(obj).forEach(n => {
      props[n] = true;
    });
    obj = Object.getPrototypeOf(obj);
  } while (obj);

  return Object.keys(props);
}

function evaluate (x) {
  try {
    return eval(x); // eslint-disable-line no-eval
  } catch (err) {
    return eval('(()=>{let x = ' + x + '; return x;})()'); // eslint-disable-line no-eval
  }
}

function safeAccess (obj, prop) {
  try {
    return obj[prop];
  } catch (e) {
    return '[Error] ' + e;
  }
}

function displayReady (x, visited = []) {
  visited.push(x);
  let toString = Object.prototype.toString;

  if (x === undefined) { return '<span class="undefinedType">undefined</span>'; }
  if (x === null) {
    return '<span class="undefinedType">null</span>';
  }
  if (typeof (x) === 'string') {
    return `"<span class="${typeof (x)}Type">${escapeHtml(x)}</span>"`;
  }
  if (toString.call(x) === '[object Array]') {
    return `[${x.map(el => displayReady(el, visited)).join(', ')}]`;
  }
  if (typeof (x) === 'function') { return `<span class="${typeof (x)}Type">function () { /* ... */ }</span>`; }
  if (typeof (x) === 'object') {
    let constructor = x.constructor || Object;
    let objToString = typeof (x.toString) === 'function' ? x.toString : toString;

    if (objToString.call(x).indexOf('[object ') !== 0) {
      return `<span class="${constructor.name}Object">${escapeHtml(objToString.call(x))}</span>`;
    }

    return `
      <span class="objectType ${constructor.name}Object">
        ${constructor.name} {
          ${Object.keys(x).filter(f => visited.indexOf(safeAccess(x, f)) === -1).map(f => `
            <span class="fieldType">${escapeHtml(f)}</span>: ${displayReady(safeAccess(x, f), visited)}
          `).join(', ')}
        }
      </span>
    `;
  }
  return `<span class="${typeof (x)}Type">${escapeHtml(JSON.stringify(x))}</span>`;
}

if (!localStorage.history) {
  localStorage.history = '[]';
}
window.historyData = JSON.parse(localStorage.history);
window.historyIndex = window.historyData.length;
if (!localStorage.watches) {
  localStorage.watches = '[]';
}
window.watches = {};

function watch (name, f) {
  let status = document.getElementById('status');
  let cleanName = name.replace(/[^a-zA-Z0-9]/, '');

  status.innerHTML += `
    <div class="watch" id="watch_${cleanName}">
      <span class="expr" id="expr_${cleanName}">${escapeHtml(name)}</span>
      <span class="res" id="res_${cleanName}"></span>
    </div>
  `;
  window.watches[name] = f;
}

let savedWatches = JSON.parse(localStorage.watches);

savedWatches.forEach(w => watch(w[1], () => evaluate(w[0])));

if (typeof (window.web3) === 'object' && window.watches.latest === undefined) {
  watch('latest', () => window.web3.eth.blockNumber);
}

function refreshWatches () {
  for (let n in window.watches) {
    let r = window.watches[n]();
    let cn = n.replace(/[^a-zA-Z0-9]/, '');
    let e = document.getElementById(`res_${cn}`);

    if (typeof (r) === 'object' && r.then && r.then.call) {
      r.then(r => {
        e.innerHTML = displayReady(r);
      });
    } else {
      e.innerHTML = displayReady(r);
    }
  }
}

function removeWatch (name) {
  let e = document.getElementById(`watch_${name}`);

  e.parentNode.removeChild(e);
  delete window.watches[name];
}

function newLog (level, text) {
  let icon = {
    debug: '&nbsp;',
    log: '&nbsp;',
    warn: '⚠',
    error: '✖',
    info: 'ℹ'
  };

  pushLine([
    '<div class="entry log ',
    level,
    'Level"><span class="type">',
    icon[level],
    '</span><span class="text">',
    escapeHtml(text),
    '</span></div>'
  ].join(''));
}

function exec () {
  let command = document.getElementById('command');
  let c = command.value;

  if (c !== '') {
    command.value = '';
    window.historyData.push(c);
    while (window.historyData.length > 1000) {
      window.historyData.shift();
    }

    localStorage.history = JSON.stringify(window.historyData);
    window.historyIndex = window.historyData.length;

    if (c.indexOf('//') === 0) {
      let n = c.substr(2);

      savedWatches = savedWatches.filter(x => x[1] !== n);
      localStorage.watches = JSON.stringify(savedWatches);
      removeWatch(n);
    } else if (c.indexOf('//') !== -1) {
      let x = c.split('//');
      let e = x[0];

      savedWatches.push(x);
      localStorage.watches = JSON.stringify(savedWatches);
      watch(x[1], () => evaluate(e));

      pushLine([
        '<div class="entry command"><span class="type">&gt;</span><span class="text">',
        escapeHtml(c),
        '</span></div>'
      ].join(''));

      pushLine([
        '<div class="entry addwatch"><span class="type">✓</span><span class="text">',
        'Watch added',
        '</span></div>'
      ].join(''));
    } else {
      pushLine([
        '<div class="entry command"><span class="type">&gt;</span><span class="text">',
        escapeHtml(c),
        '</span></div>'
      ].join(''));

      let res;

      try {
        res = evaluate(c);
        if (typeof (res) === 'object' && res !== null && typeof res.then === 'function') {
          let id = window.historyData.length;

          pushLine([
            '<div class="entry result"><span class="type">&lt;</span><span class="text" id="pending',
            id,
            '">...</span></div>'
          ].join(''));

          res.then(r => {
            document.getElementById('pending' + id).innerHTML = displayReady(r);
          });
        } else {
          pushLine([
            '<div class="entry result"><span class="type">&lt;</span><span class="text">',
            displayReady(res),
            '</span></div>'
          ].join(''));
        }
      } catch (err) {
        pushLine([
          '<div class="entry error"><span class="type">✖</span><span class="text">Unhandled exception: ',
          escapeHtml(err.message),
          '</span></div>'
        ]);
      }
    }
  }

  refreshWatches();
}

function pushLine (l) {
  document.getElementById('history').innerHTML += l;
  let h = document.getElementById('history-wrap');

  h.scrollTop = h.scrollHeight;
}

let autocompletes = [];
let currentAuto = null;
let currentPots = [];
let currentStem = null;

function updateAutocomplete () {
  let v = document.getElementById('command').value;

  if (!v.length) {
    cancelAutocomplete();
    return;
  }

  let t = v.split('.');
  let last = t.pop();
  let tj = t.join('.');
  let ex = t.length > 0 ? tj : 'window';

  if (currentStem !== tj) {
    autocompletes = getAllPropertyNames(evaluate(ex));
    currentStem = tj;
  }

  let dl = document.getElementById('autocomplete');

  currentPots = autocompletes.filter(n => n.startsWith(last));
  if (currentPots.length > 0) {
    if (currentPots.indexOf(currentAuto) === -1) {
      currentAuto = currentPots[0];
    }

    dl.innerHTML = currentPots
      .map((n, i) => `
        <div id="pot${i}" class="${currentAuto === n ? 'ac-selected' : 'ac-unselected'}">
          <span class="ac-already">${escapeHtml(last)}</span><span class="ac-new">${escapeHtml(n.substr(last.length))}</span>
        </div>`
      )
      .join('');
    dl.hidden = false;
  } else {
    cancelAutocomplete();
  }
}

function enactAutocomplete () {
  if (currentAuto != null) {
    document.getElementById('command').value = (currentStem !== '' ? currentStem + '.' : '') + currentAuto;
    cancelAutocomplete();
  }
}

function cancelAutocomplete () {
  document.getElementById('autocomplete').hidden = true;
  currentAuto = null;
}

function scrollAutocomplete (positive) {
  if (currentAuto != null) {
    let i = currentPots.indexOf(currentAuto);

    document.getElementById('pot' + i).classList = ['ac-unselected'];
    if (positive && i < currentPots.length - 1) {
      ++i;
    } else if (!positive && i > 0) {
      --i;
    }
    currentAuto = currentPots[i];
    let sel = document.getElementById('pot' + i);

    sel.classList = ['ac-selected'];
    sel.scrollIntoViewIfNeeded();
  }
}

document.getElementById('command').addEventListener('paste', updateAutocomplete);
document.getElementById('command').addEventListener('input', updateAutocomplete);
document.getElementById('command').addEventListener('focusout', cancelAutocomplete);
document.getElementById('command').addEventListener('blur', cancelAutocomplete);

document.getElementById('command').addEventListener('keydown', function (event) {
  let el = document.getElementById('command');

  if (currentAuto != null) {
    if (event.keyCode === 38 || event.keyCode === 40) {
      event.preventDefault();
      scrollAutocomplete(event.keyCode === 40);
    } else if ((event.keyCode === 39 || event.keyCode === 9 || event.keyCode === 13) && el.selectionStart === el.value.length) {
      event.preventDefault();
      enactAutocomplete();
    } else if (event.keyCode === 27) {
      event.preventDefault();
      cancelAutocomplete();
    }
  } else {
    let command = document.getElementById('command');

    if (event.keyCode === 38 && window.historyIndex > 0) {
      event.preventDefault();
      window.historyIndex--;
      command.value = window.historyData[window.historyIndex];
    }
    if (event.keyCode === 40 && window.historyIndex < window.historyData.length) {
      event.preventDefault();
      window.historyIndex++;
      command.value = window.historyIndex < window.historyData.length ? window.historyData[window.historyIndex] : '';
    }
  }

  if (event.keyCode >= 48 || event.keyCode === 8) {
    let t = document.getElementById('command').value;

    setTimeout(() => {
      if (t !== document.getElementById('command').value) {
        updateAutocomplete();
      }
    }, 0);
  } else {
    setTimeout(() => {
      if (el.selectionStart !== el.value.length) {
        cancelAutocomplete();
      }
    }, 0);
  }
});

document.getElementById('command').addEventListener('keyup', function (event) {
  if (event.keyCode === 13) {
    event.preventDefault();
    exec();
  }
});

document.getElementById('command').focus();
if (typeof (web3) === 'object') {
  window.web3 = web3;
}
refreshWatches();

['debug', 'error', 'info', 'log', 'warn'].forEach(n => {
  let old = window.console[n].bind(window.console);

  window.console[n] = x => {
    old(x);
    newLog(n, x);
  };
});

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// /// Home comforts.

if (typeof (web3) === 'object') {
  // Usage example:
  // web3.eth.traceCall({
  //     to: theChicken.address,
  //     data: theChicken.withdraw.getData(100000000000000000),
  //     gas: 100000
  //   },
  //   `["trace", "vmTrace", "stateDiff"]
  //  )
  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'traceCall',
        call: 'trace_call',
        params: 2,
        inputFormatter: [web3._extend.formatters.inputCallFormatter, null]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'traceSendRawTransaction',
        call: 'trace_rawTransaction',
        params: 2,
        inputFormatter: [null, null]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'traceReplayTransaction',
        call: 'trace_replayTransaction',
        params: 2,
        inputFormatter: [null, null]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'setMode',
        call: 'parity_setMode',
        params: 1
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'mode',
        call: 'parity_mode',
        params: 0
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'traceTransaction',
        call: 'trace_Transaction',
        params: 1,
        inputFormatter: [null]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'gasPriceStatistics',
        call: 'parity_gasPriceStatistics',
        params: 0,
        outputFormatter: function (a) { return a.map(web3.toBigNumber); }
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'registryAddress',
        call: 'parity_registryAddress',
        params: 0
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'accountsInfo',
        call: 'personal_accountsInfo',
        outputFormatter: function (m) {
          Object.keys(m).forEach(k => {
            m[k].meta = JSON.parse(m[k].meta);
            m[k].meta.name = m[k].name;
            m[k].meta.uuid = m[k].uuid;
            m[k] = m[k].meta;
          }); return m;
        },
        params: 0
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'setAccountName',
        call: 'personal_setAccountName',
        params: 2
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'setAccountMeta',
        call: 'personal_setAccountMeta',
        params: 2,
        inputFormatter: [a => a, JSON.stringify]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'postTransaction',
        call: 'eth_postTransaction',
        params: 1,
        inputFormatter: [web3._extend.formatters.inputCallFormatter]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'postSign',
        call: 'eth_postSign',
        params: 1
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'encryptMessage',
        call: 'parity_encryptMessage',
        params: 2
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'checkRequest',
        call: 'eth_checkRequest',
        params: 1
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'listAccounts',
        call: 'parity_listAccounts',
        params: 0
      })
    ]
  });

  {
    let postTransaction = web3.eth.postTransaction.bind(web3.eth);
    let sendTransaction = web3.eth.sendTransaction.bind(web3.eth);

    web3.eth.sendTransaction = function (options, f) {
      // No callback - do sync API.
      if (typeof f !== 'function') {
        return sendTransaction(options);
      }
      // Callback - use async API.
      let id = postTransaction(options);

      console.log('Posted trasaction id=' + id);
      let timerId = window.setInterval(check, 500);

      function check () {
        try {
          let r = web3.eth.checkRequest(id);

          if (typeof r === 'string') {
            clearInterval(timerId);
            if (r === '0x0000000000000000000000000000000000000000000000000000000000000000') {
              f('Rejected', r);
            } else {
              f(null, r);
            }
          } else if (r !== null) {
            console.log('checkRequest returned: ' + r);
          }
        } catch (e) {
          clearInterval(timerId);
          f('Rejected', null);
        }
      }
    };
  }

  web3.eth.installInterceptor = function (interceptor) {
    let oldSendTransaction = web3.eth.sendTransaction.bind(web3.eth);

    web3.eth.sendTransaction = function (options, f) {
      if (!interceptor(options)) {
        return '0x0000000000000000000000000000000000000000000000000000000000000000';
      }

      return oldSendTransaction(options, f);
    };
  };

  web3.eth.reporter = function (e, r) {
    if (e) {
      console.log('Error confirming transaction: ' + e);
    } else {
      let addr = r;
      let confirmed = false;
      let timerId = window.setInterval(function check () {
        let receipt = web3.eth.getTransactionReceipt(addr);

        if (receipt != null) {
          if (!confirmed) {
            console.log('Transaction confirmed (' + r + '); used ' + receipt.gasUsed + ' gas; left ' + receipt.logs.length + ' logs; mining...');
            confirmed = true;
          }
          if (typeof receipt.blockHash === 'string') {
            clearInterval(timerId);
            console.log('Mined into block ' + receipt.blockNumber);
          }
        }
      }, 500);
    }
  };

  {
    let oldSha3 = web3.sha3;

    web3.sha3 = function (data, format) {
      if (typeof format !== 'string' || (format !== 'hex' && format !== 'bin')) {
        format = data.startsWith('0x') ? 'hex' : 'bin';
      }
      return oldSha3(data, { encoding: format });
    };
  }

  {
    let Registry = web3.eth.contract([{ 'constant': false, 'inputs': [{ 'name': '_new', 'type': 'address' }], 'name': 'setOwner', 'outputs': [], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'string' }], 'name': 'confirmReverse', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }], 'name': 'reserve', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }, { 'name': '_value', 'type': 'bytes32' }], 'name': 'set', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }], 'name': 'drop', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }], 'name': 'getAddress', 'outputs': [{ 'name': '', 'type': 'address' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_amount', 'type': 'uint256' }], 'name': 'setFee', 'outputs': [], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_to', 'type': 'address' }], 'name': 'transfer', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': true, 'inputs': [], 'name': 'owner', 'outputs': [{ 'name': '', 'type': 'address' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }], 'name': 'reserved', 'outputs': [{ 'name': 'reserved', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [], 'name': 'drain', 'outputs': [], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'string' }, { 'name': '_who', 'type': 'address' }], 'name': 'proposeReverse', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }], 'name': 'getUint', 'outputs': [{ 'name': '', 'type': 'uint256' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }], 'name': 'get', 'outputs': [{ 'name': '', 'type': 'bytes32' }], 'type': 'function' }, { 'constant': true, 'inputs': [], 'name': 'fee', 'outputs': [{ 'name': '', 'type': 'uint256' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '', 'type': 'address' }], 'name': 'reverse', 'outputs': [{ 'name': '', 'type': 'string' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }, { 'name': '_value', 'type': 'uint256' }], 'name': 'setUint', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [], 'name': 'removeReverse', 'outputs': [], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }, { 'name': '_value', 'type': 'address' }], 'name': 'setAddress', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'anonymous': false, 'inputs': [{ 'indexed': false, 'name': 'amount', 'type': 'uint256' }], 'name': 'Drained', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': false, 'name': 'amount', 'type': 'uint256' }], 'name': 'FeeChanged', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'bytes32' }, { 'indexed': true, 'name': 'owner', 'type': 'address' }], 'name': 'Reserved', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'bytes32' }, { 'indexed': true, 'name': 'oldOwner', 'type': 'address' }, { 'indexed': true, 'name': 'newOwner', 'type': 'address' }], 'name': 'Transferred', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'bytes32' }, { 'indexed': true, 'name': 'owner', 'type': 'address' }], 'name': 'Dropped', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'bytes32' }, { 'indexed': true, 'name': 'owner', 'type': 'address' }, { 'indexed': true, 'name': 'key', 'type': 'string' }, { 'indexed': false, 'name': 'plainKey', 'type': 'string' }], 'name': 'DataChanged', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'string' }, { 'indexed': true, 'name': 'reverse', 'type': 'address' }], 'name': 'ReverseProposed', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'string' }, { 'indexed': true, 'name': 'reverse', 'type': 'address' }], 'name': 'ReverseConfirmed', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'string' }, { 'indexed': true, 'name': 'reverse', 'type': 'address' }], 'name': 'ReverseRemoved', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'old', 'type': 'address' }, { 'indexed': true, 'name': 'current', 'type': 'address' }], 'name': 'NewOwner', 'type': 'event' }]);

    web3.eth.registry = Registry.at(web3.eth.registryAddress());
    web3.eth.registry.lookup = (name, field) => web3.eth.registry.get(web3.sha3(name), field);
    web3.eth.registry.lookupAddress = (name, field) => web3.eth.registry.getAddress(web3.sha3(name), field);
    web3.eth.registry.lookupUint = (name, field) => web3.eth.registry.getUint(web3.sha3(name), field);

    let TokenReg = web3.eth.contract([{ 'constant': true, 'inputs': [{ 'name': '_id', 'type': 'uint256' }], 'name': 'token', 'outputs': [{ 'name': 'addr', 'type': 'address' }, { 'name': 'tla', 'type': 'string' }, { 'name': 'base', 'type': 'uint256' }, { 'name': 'name', 'type': 'string' }, { 'name': 'owner', 'type': 'address' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_new', 'type': 'address' }], 'name': 'setOwner', 'outputs': [], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_addr', 'type': 'address' }, { 'name': '_tla', 'type': 'string' }, { 'name': '_base', 'type': 'uint256' }, { 'name': '_name', 'type': 'string' }], 'name': 'register', 'outputs': [{ 'name': '', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_fee', 'type': 'uint256' }], 'name': 'setFee', 'outputs': [], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_id', 'type': 'uint256' }, { 'name': '_key', 'type': 'bytes32' }], 'name': 'meta', 'outputs': [{ 'name': '', 'type': 'bytes32' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_addr', 'type': 'address' }, { 'name': '_tla', 'type': 'string' }, { 'name': '_base', 'type': 'uint256' }, { 'name': '_name', 'type': 'string' }, { 'name': '_owner', 'type': 'address' }], 'name': 'registerAs', 'outputs': [{ 'name': '', 'type': 'bool' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_tla', 'type': 'string' }], 'name': 'fromTLA', 'outputs': [{ 'name': 'id', 'type': 'uint256' }, { 'name': 'addr', 'type': 'address' }, { 'name': 'base', 'type': 'uint256' }, { 'name': 'name', 'type': 'string' }, { 'name': 'owner', 'type': 'address' }], 'type': 'function' }, { 'constant': true, 'inputs': [], 'name': 'owner', 'outputs': [{ 'name': '', 'type': 'address' }], 'type': 'function' }, { 'constant': false, 'inputs': [], 'name': 'drain', 'outputs': [], 'type': 'function' }, { 'constant': true, 'inputs': [], 'name': 'tokenCount', 'outputs': [{ 'name': '', 'type': 'uint256' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_id', 'type': 'uint256' }], 'name': 'unregister', 'outputs': [], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_addr', 'type': 'address' }], 'name': 'fromAddress', 'outputs': [{ 'name': 'id', 'type': 'uint256' }, { 'name': 'tla', 'type': 'string' }, { 'name': 'base', 'type': 'uint256' }, { 'name': 'name', 'type': 'string' }, { 'name': 'owner', 'type': 'address' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_id', 'type': 'uint256' }, { 'name': '_key', 'type': 'bytes32' }, { 'name': '_value', 'type': 'bytes32' }], 'name': 'setMeta', 'outputs': [], 'type': 'function' }, { 'constant': true, 'inputs': [], 'name': 'fee', 'outputs': [{ 'name': '', 'type': 'uint256' }], 'type': 'function' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'tla', 'type': 'string' }, { 'indexed': true, 'name': 'id', 'type': 'uint256' }, { 'indexed': false, 'name': 'addr', 'type': 'address' }, { 'indexed': false, 'name': 'name', 'type': 'string' }], 'name': 'Registered', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'tla', 'type': 'string' }, { 'indexed': true, 'name': 'id', 'type': 'uint256' }], 'name': 'Unregistered', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'id', 'type': 'uint256' }, { 'indexed': true, 'name': 'key', 'type': 'bytes32' }, { 'indexed': false, 'name': 'value', 'type': 'bytes32' }], 'name': 'MetaChanged', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'old', 'type': 'address' }, { 'indexed': true, 'name': 'current', 'type': 'address' }], 'name': 'NewOwner', 'type': 'event' }]);

    web3.eth.tokenReg = TokenReg.at(web3.eth.registry.lookupAddress('tokenreg', 'A'));
  }
}

