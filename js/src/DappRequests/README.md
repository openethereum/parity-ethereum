# Terminology used

To be clear with the terminology used in the code here:
- a *method* is a JSON-RPC api method (see here: https://github.com/paritytech/parity/wiki/JSONRPC-parity_accounts-module) OR a shell method
- a *methodGroup* is the grouping of similar methods (see `methodGroups.js`)
- a *permission* is a boolean which tells if an app is allowed to call a method or not
- a *request* is when an app prompts the shell to call a method (has a `source`, an `origin` and some `data`)
- a *requestGroup* is an array of *requests* whose methods are in the same *methodGroup*
