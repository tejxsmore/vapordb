<script lang="ts">

  let key = $state('');
  let value = $state('');
  let ttl = $state(60);
  let command = $state('get');
  let response = $state(null);
  let loading = $state(false);

  const API_URL = 'http://127.0.0.1:3030/cmd';

  async function runCommand() {
    loading=true;
    response=null;

    let payload;
    switch (command) {
      case 'get':
        payload = { cmd: 'get', key };
        break;
      case 'set':
        payload = { cmd: 'set', key, value };
        break;
      case 'del':
        payload = { cmd: 'del', key };
        break;
      case 'setwithexpiration':
        payload = { cmd: 'setwithexpiration', key, value, ttl_secs: ttl };
        break;
      default:
        payload = null;
    }

     if (!payload) {
      console.error('Invalid command', 'error');
      loading = false;
      return;
    }

    try {
      let res = await fetch(API_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      let data = await res.json();

      if (data.error) {
        console.error(`Error: ${data.error}`, 'error');
      } else {
        console.log('Success', 'success');
        response = data.result ?? 'None';
      }
    } catch (e: any) {
      console.error(`Network error: ${e.message}`, 'error');
    }

    console.log('Command executed:', command, 'Key:', key, 'Value:', value, 'TTL:', ttl);
    console.log('Response:', response);
    loading = false;
  }  
</script>

<section class="space-y-8 max-w-xl mx-auto mt-12 px-4">
  <h2 class="text-yellow font-semibold text-3xl tracking-wide select-none">VaporDB Interface</h2>

  <div class="flex flex-col space-y-4">
    <label for="command" class="text-yellow font-medium">Command</label>
    <select bind:value={command} name="command" class="bg-dark border border-yellow text-yellow rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-yellow">
      <option value="get">GET</option>
      <option value="set">SET</option>
      <option value="del">DEL</option>
      <option value="setwithexpiration">SET with Expiration</option>
    </select>
  </div>

  <div class="flex flex-col space-y-4">
    <label for="key" class="text-yellow font-medium">Key</label>
    <input
      type="text"
      name="key"
      bind:value={key}
      placeholder="Enter key"
      class="bg-dark border border-yellow rounded px-3 py-2 text-yellow focus:outline-none focus:ring-2 focus:ring-yellow"
    />
  </div>

  {#if command === 'set' || command === 'setwithexpiration'}
    <div class="flex flex-col space-y-4">
      <label for="value" class="text-yellow font-medium">Value</label>
      <input
        type="text"
        name="value"
        bind:value={value}
        placeholder="Enter value"
        class="bg-dark border border-yellow rounded px-3 py-2 text-yellow focus:outline-none focus:ring-2 focus:ring-yellow"
      />
    </div>
  {/if}

  {#if command === 'setwithexpiration'}
    <div class="flex flex-col space-y-4">
      <label for="ttl" class="text-yellow font-medium">TTL (seconds)</label>
      <input
        type="number"
        name="ttl"
        bind:value={ttl}
        min="1"
        placeholder="Enter TTL in seconds"
        class="bg-dark border border-yellow rounded px-3 py-2 text-yellow focus:outline-none focus:ring-2 focus:ring-yellow"
      />
    </div>
  {/if}

  <button
    onclick={runCommand}
    class="bg-yellow text-dark font-bold px-6 py-2 rounded hover:bg-yellow/90 active:scale-95 transition transform select-none disabled:opacity-50 disabled:cursor-not-allowed"
    disabled={loading || !key || ((command === 'set' || command === 'setwithexpiration') && !value) || (command === 'setwithexpiration' && (!ttl || ttl < 1))}
  >
    {loading ? 'Running...' : 'Execute'}
  </button>

  {#if response === null && !loading}
    <div class="mt-6 text-yellow">No response yet. Execute a command to see results.</div>
  {/if}
</section>