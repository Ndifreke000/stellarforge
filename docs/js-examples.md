# JS/TS Examples — StellarForge Contracts

TypeScript examples for invoking StellarForge Soroban contracts using the
[Stellar JavaScript SDK](https://github.com/stellar/js-stellar-sdk).

## Prerequisites

- **Node.js** ≥ 18
- **@stellar/stellar-sdk** ≥ 13.0.0

```bash
npm install @stellar/stellar-sdk
```

- **Network**: All examples target Stellar testnet. The RPC endpoint is:
  `https://soroban-testnet.stellar.org`
- **Funded account**: You need a funded testnet keypair. Generate one with:

```bash
stellar keys generate my-key --network testnet --fund
```

## Transaction Flow

Every example follows the same Soroban simulate-then-submit pattern:

```
build operation
      │
      ▼
simulateTransaction  ──► error? → throw Error
      │ (success)
      ▼
assembleTransaction  (applies resource fee + ledger footprint)
      │
      ▼
sign with Keypair
      │
      ▼
sendTransaction
      │
      ▼
poll getTransaction until SUCCESS or FAILED
      │
      ▼
decode return ScVal → typed result
```

---

## forge-vesting — `claim()`

Withdraws all currently vested and unclaimed tokens for the beneficiary.
The caller must be the beneficiary of the vesting contract.

```typescript
import {
  Keypair,
  Networks,
  TransactionBuilder,
  BASE_FEE,
  Contract,
  SorobanRpc,
  scValToNative,
} from "@stellar/stellar-sdk";

// Error codes from VestingError enum in forge-vesting
const VESTING_ERRORS: Record<number, string> = {
  2: "Vesting contract is not initialized",
  4: "Cliff period has not been reached yet",
  5: "No tokens available to claim",
  6: "Vesting schedule has been cancelled",
};

async function claimVesting(
  keypair: Keypair,
  contractId: string,
  rpcUrl: string
): Promise<bigint> {
  const server = new SorobanRpc.Server(rpcUrl);

  // Load the current account state (sequence number, etc.)
  const account = await server.getAccount(keypair.publicKey());

  // Build the contract operation — claim() takes no arguments beyond the env
  const contract = new Contract(contractId);
  const operation = contract.call("claim");

  // Wrap the operation in a transaction
  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.TESTNET,
  })
    .addOperation(operation)
    .setTimeout(30)
    .build();

  // Simulate to get resource fee and ledger footprint
  const simResult = await server.simulateTransaction(tx);

  if (SorobanRpc.Api.isSimulationError(simResult)) {
    // Decode the contract error code and throw a descriptive message
    const code = (simResult as any)?.error
      ?.split("Error(Contract, #")[1]
      ?.split(")")[0];
    const msg = VESTING_ERRORS[Number(code)] ?? `Contract error code ${code}`;
    throw new Error(msg);
  }

  // Apply the simulated resource fee and footprint to the transaction
  const assembledTx = SorobanRpc.assembleTransaction(tx, simResult).build();

  // Sign with the beneficiary keypair
  assembledTx.sign(keypair);

  // Submit the transaction
  const sendResult = await server.sendTransaction(assembledTx);
  if (sendResult.status === "ERROR") {
    throw new Error(`Send failed: ${sendResult.errorResult}`);
  }

  // Poll until the transaction is confirmed or failed
  let getResult = await server.getTransaction(sendResult.hash);
  while (getResult.status === SorobanRpc.Api.GetTransactionStatus.NOT_FOUND) {
    await new Promise((r) => setTimeout(r, 1000));
    getResult = await server.getTransaction(sendResult.hash);
  }

  if (getResult.status === SorobanRpc.Api.GetTransactionStatus.FAILED) {
    throw new Error(`Transaction failed: ${getResult.resultXdr}`);
  }

  // Decode the returned i128 (amount claimed) and return as bigint
  const returnVal = getResult.returnValue!;
  return scValToNative(returnVal) as bigint;
}
```

---

## forge-stream — `create_stream()`

Creates a new pay-per-second token stream from a sender to a recipient.
The sender must have approved the contract to pull `ratePerSecond * durationSeconds` tokens.
Returns the new stream ID.

```typescript
import {
  Keypair,
  Networks,
  TransactionBuilder,
  BASE_FEE,
  Contract,
  SorobanRpc,
  nativeToScVal,
  scValToNative,
} from "@stellar/stellar-sdk";

// Error codes from StreamError enum in forge-stream
const STREAM_ERRORS: Record<number, string> = {
  5: "Invalid stream config: rate and duration must be > 0",
};

async function createStream(
  keypair: Keypair,       // sender — must sign and fund the stream
  contractId: string,    // deployed forge-stream contract address
  tokenAddress: string,  // SEP-41 token contract address
  recipientAddress: string,
  ratePerSecond: bigint, // tokens unlocked per second (i128)
  durationSeconds: bigint, // stream duration in seconds (u64)
  rpcUrl: string
): Promise<bigint> {     // returns the new stream ID
  const server = new SorobanRpc.Server(rpcUrl);

  const account = await server.getAccount(keypair.publicKey());

  const contract = new Contract(contractId);

  // Encode each argument to the correct Soroban ScVal type
  const operation = contract.call(
    "create_stream",
    nativeToScVal(keypair.publicKey(), { type: "address" }), // sender
    nativeToScVal(tokenAddress, { type: "address" }),         // token
    nativeToScVal(recipientAddress, { type: "address" }),     // recipient
    nativeToScVal(ratePerSecond, { type: "i128" }),           // rate_per_second
    nativeToScVal(durationSeconds, { type: "u64" })           // duration_seconds
  );

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.TESTNET,
  })
    .addOperation(operation)
    .setTimeout(30)
    .build();

  // Simulate to obtain resource fee and ledger footprint
  const simResult = await server.simulateTransaction(tx);

  if (SorobanRpc.Api.isSimulationError(simResult)) {
    const code = (simResult as any)?.error
      ?.split("Error(Contract, #")[1]
      ?.split(")")[0];
    const msg = STREAM_ERRORS[Number(code)] ?? `Contract error code ${code}`;
    throw new Error(msg);
  }

  // Assemble: attach the simulated resource fee and footprint
  const assembledTx = SorobanRpc.assembleTransaction(tx, simResult).build();

  // Sign with the sender keypair
  assembledTx.sign(keypair);

  const sendResult = await server.sendTransaction(assembledTx);
  if (sendResult.status === "ERROR") {
    throw new Error(`Send failed: ${sendResult.errorResult}`);
  }

  // Poll until confirmed
  let getResult = await server.getTransaction(sendResult.hash);
  while (getResult.status === SorobanRpc.Api.GetTransactionStatus.NOT_FOUND) {
    await new Promise((r) => setTimeout(r, 1000));
    getResult = await server.getTransaction(sendResult.hash);
  }

  if (getResult.status === SorobanRpc.Api.GetTransactionStatus.FAILED) {
    throw new Error(`Transaction failed: ${getResult.resultXdr}`);
  }

  // Decode the returned u64 stream ID and return as bigint
  const returnVal = getResult.returnValue!;
  return scValToNative(returnVal) as bigint;
}
```

---

## forge-multisig — `propose()`

Submits a treasury transfer proposal. The caller must be one of the multisig owners.
The proposer automatically counts as the first approval.
Returns the new proposal ID.

```typescript
import {
  Keypair,
  Networks,
  TransactionBuilder,
  BASE_FEE,
  Contract,
  SorobanRpc,
  nativeToScVal,
  scValToNative,
} from "@stellar/stellar-sdk";

// Error codes from MultisigError enum in forge-multisig
const MULTISIG_ERRORS: Record<number, string> = {
  2:  "Multisig contract is not initialized",
  3:  "Caller is not an owner of this multisig",
  11: "Proposal amount must be greater than zero",
};

async function proposeTransfer(
  keypair: Keypair,      // proposer — must be a registered owner
  contractId: string,   // deployed forge-multisig contract address
  tokenAddress: string, // token to transfer from the treasury
  toAddress: string,    // destination address for the transfer
  amount: bigint,       // amount to transfer (i128)
  rpcUrl: string
): Promise<bigint> {    // returns the new proposal ID
  const server = new SorobanRpc.Server(rpcUrl);

  const account = await server.getAccount(keypair.publicKey());

  const contract = new Contract(contractId);

  // Encode each argument to the correct Soroban ScVal type
  const operation = contract.call(
    "propose",
    nativeToScVal(keypair.publicKey(), { type: "address" }), // proposer
    nativeToScVal(toAddress, { type: "address" }),            // to
    nativeToScVal(tokenAddress, { type: "address" }),         // token
    nativeToScVal(amount, { type: "i128" })                   // amount
  );

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.TESTNET,
  })
    .addOperation(operation)
    .setTimeout(30)
    .build();

  // Simulate to obtain resource fee and ledger footprint
  const simResult = await server.simulateTransaction(tx);

  if (SorobanRpc.Api.isSimulationError(simResult)) {
    const code = (simResult as any)?.error
      ?.split("Error(Contract, #")[1]
      ?.split(")")[0];
    const msg = MULTISIG_ERRORS[Number(code)] ?? `Contract error code ${code}`;
    throw new Error(msg);
  }

  // Assemble: attach the simulated resource fee and footprint
  const assembledTx = SorobanRpc.assembleTransaction(tx, simResult).build();

  // Sign with the proposer keypair
  assembledTx.sign(keypair);

  const sendResult = await server.sendTransaction(assembledTx);
  if (sendResult.status === "ERROR") {
    throw new Error(`Send failed: ${sendResult.errorResult}`);
  }

  // Poll until confirmed
  let getResult = await server.getTransaction(sendResult.hash);
  while (getResult.status === SorobanRpc.Api.GetTransactionStatus.NOT_FOUND) {
    await new Promise((r) => setTimeout(r, 1000));
    getResult = await server.getTransaction(sendResult.hash);
  }

  if (getResult.status === SorobanRpc.Api.GetTransactionStatus.FAILED) {
    throw new Error(`Transaction failed: ${getResult.resultXdr}`);
  }

  // Decode the returned u64 proposal ID and return as bigint
  const returnVal = getResult.returnValue!;
  return scValToNative(returnVal) as bigint;
}
```
