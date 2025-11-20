import { useState } from 'react';
import { Server } from 'stellar-sdk';

export default function Home() {
  const [trace, setTrace] = useState(null);
  const server = new Server('https://soroban-testnet.stellar.org');

  const mintResource = async () => {
    // Integra Freighter para sign (Wallet A)
    // Llama RPC: invoke contract con params
  };

  const queryTrace = async (id) => {
    // Fetch via Stellar SDK
    setTrace(/* result */);
  };

  return (
    <div>
      <h1>Block's Resources Trace</h1>
      <button onClick={() => mintResource()}>Mint Agua Amazonas</button>
      <button onClick={() => queryTrace(1)}>Query ID 1</button>
      {trace && <pre>{JSON.stringify(trace, null, 2)}</pre>}
    </div>
  );
}
