# wasmcloud-keyvalue-cdc

This is a proof of concept "middleware" component to act as a change data capture for the wasi key-value interface. It intercepts incoming get, set and delete functions from the wasi-keyvalue interface and then publishes them to a wasi-messaging interface.

## Example Application

This application (`wadm.yaml`) demonstrates the capabilities of the keyvalue-cdc component. Here it sits between the `http-keyvalue-counter-rust` example component and the kv provider (in this case the `provider-keyvalue-redis`).

```
┌────────────────┐                  ┌───────────┐
│                │                  │           │
│ HTTP provider  ├──────────────────► http      │
│                │                  │           │
└────────────────┘                  │ keyvalue  │
                                    │           │
┌────────────────┐   ┌──────────┐   │ counter   │
│                │   │          │   │           │
│ kv provider    ◄───┤ kv cdc   ◄───┤ rust      │
│                │   │          │   │           │
└────────────────┘   └────┬─────┘   └───────────┘
                          │                      
 ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ 
                          │                      
┌────────────────┐        │         ┌───────────┐
│                ◄────────┘         │           │
│ msg provider   │                  │ msg sync  │
│                ├──────────────────►           │
└────────────────┘                  └───────────┘
```