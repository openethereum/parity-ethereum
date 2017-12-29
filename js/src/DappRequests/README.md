# Terminology used

To be clear with the terminology used in the code here:

- a *method* is an allowed JSON-RPC api method or a shell method
- a *methodGroup* is the grouping of similar methods (see `methodGroups.js`)
- a *permission* is a boolean which tells if an app is allowed to call a method or not
- a *request* is when an app prompts the shell to call a method
- a *requestGroup* is a map of the following form
```javascript
{
  appId1: {
    methodGroup1: [request1, request2] // This is a requestGroup sub-item
  },
  appId2: {
    methodGroup1: [request1]
    methodGroup2: [request3]
  },
  // ...
}
```
