<script lang="ts">
    let command = $state('get');
    let dataType = $state('string');
    let key = $state('');
    let value = $state('');
    let field = $state('');
    let ttl = $state(60);
    let start = $state(0);
    let end = $state(-1);
    let response: {
        type: 'success' | 'error';
        content: string;
    } | null = $state(null);
    let loading = $state(false);
    let lastCommand = $state('');

    const API_URL = 'http://127.0.0.1:3030/cmd';

    const commands = {
        string: [
            { value: 'get', label: 'GET', description: 'Get value by key' },
            { value: 'set', label: 'SET', description: 'Set key-value pair' },
            { value: 'del', label: 'DEL', description: 'Delete key' },
            { value: 'setwithexpiration', label: 'SET with TTL', description: 'Set with expiration' }
        ],
        hash: [
            { value: 'hset', label: 'HSET', description: 'Set hash field' },
            { value: 'hget', label: 'HGET', description: 'Get hash field' },
            { value: 'hdel', label: 'HDEL', description: 'Delete hash field' }
        ],
        list: [
            { value: 'lpush', label: 'LPUSH', description: 'Push to list head' },
            { value: 'rpush', label: 'RPUSH', description: 'Push to list tail' },
            { value: 'lpop', label: 'LPOP', description: 'Pop from list head' },
            { value: 'rpop', label: 'RPOP', description: 'Pop from list tail' },
            { value: 'lrange', label: 'LRANGE', description: 'Get list range' }
        ],
        set: [
            { value: 'sadd', label: 'SADD', description: 'Add to set' },
            { value: 'srem', label: 'SREM', description: 'Remove from set' },
            { value: 'smembers', label: 'SMEMBERS', description: 'Get all set members' }
        ]
    };

    function updateCommand() {
        const availableCommands = commands[dataType as keyof typeof commands];
        if (availableCommands && !availableCommands.find((c) => c.value === command)) {
            command = availableCommands[0].value;
        }
    }

    $effect(() => {
        updateCommand();
    });

    async function runCommand() {
        if (!key.trim()) return;
        loading = true;
        response = null;
        lastCommand = `${command.toUpperCase()} ${key}`;

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
            case 'hset':
                payload = { cmd: 'hset', key, field, value };
                break;
            case 'hget':
                payload = { cmd: 'hget', key, field };
                break;
            case 'hdel':
                payload = { cmd: 'hdel', key, field };
                break;
            case 'lpush':
                payload = { cmd: 'lpush', key, value };
                break;
            case 'rpush':
                payload = { cmd: 'rpush', key, value };
                break;
            case 'lpop':
                payload = { cmd: 'lpop', key };
                break;
            case 'rpop':
                payload = { cmd: 'rpop', key };
                break;
            case 'lrange':
                payload = { cmd: 'lrange', key, start, end };
                break;
            case 'sadd':
                payload = { cmd: 'sadd', key, value };
                break;
            case 'srem':
                payload = { cmd: 'srem', key, value };
                break;
            case 'smembers':
                payload = { cmd: 'smembers', key };
                break;
        }

        try {
            const res = await fetch(API_URL, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });
            const data = await res.json();

            if (data.error) {
                response = { type: 'error', content: data.error };
            } else {
                let content = data.result ?? 'Operation completed successfully';
                
                try {
                    const parsed = JSON.parse(content);
                    content = JSON.stringify(parsed, null, 2);
                } catch (e) {
                    // Not JSON, keep as is
                }
                
                response = { type: 'success', content };
            }
        } catch (error: any) {
            response = { type: 'error', content: `Network error: ${error.message}` };
        }

        loading = false;
    }

    function needsValue() {
        return ['set', 'setwithexpiration', 'hset', 'lpush', 'rpush', 'sadd', 'srem'].includes(command);
    }

    function needsField() {
        return ['hset', 'hget', 'hdel'].includes(command);
    }

    function needsTtl() {
        return command === 'setwithexpiration';
    }

    function needsRange() {
        return command === 'lrange';
    }

    function isFormValid() {
        if (!key.trim()) return false;
        if (needsValue() && !value.trim()) return false;
        if (needsField() && !field.trim()) return false;
        if (needsTtl() && (!ttl || ttl < 1)) return false;
        return true;
    }
</script>

<div class="min-h-screen p-6">
    <div class="text-center py-12 space-y-4">
        <h1 class="text-4xl font-bold text-(--primary)">VaporDB</h1>
        <p class="text-gray-400">In-Memory Key-Value Database Interface</p>
    </div>

    <div class="max-w-4xl mx-auto space-y-6">
        <div class="grid md:grid-cols-2 gap-6">
            <div class="bg-(--accent) border border-(--border) rounded-[12px] p-6 space-y-6">
                <div class="flex items-center gap-2 text-(--primary)">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M20.25 6.375c0 2.278-3.694 4.125-8.25 4.125S3.75 8.653 3.75 6.375m16.5 0c0-2.278-3.694-4.125-8.25-4.125S3.75 4.097 3.75 6.375m16.5 0v11.25c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125V6.375m16.5 0v3.75m-16.5-3.75v3.75m16.5 0v3.75C20.25 16.153 16.556 18 12 18s-8.25-1.847-8.25-4.125v-3.75m16.5 0c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125" />
                    </svg>
                    <span class="font-medium">Data Type</span>
                </div>
                
                <div class="space-y-3">
                    <div>
                        <label for="type" class="block text-sm text-gray-300 mb-1">Select Data Structure</label>
                        <select name="type" bind:value={dataType} class="w-full bg-(--border) rounded-[6px] px-3 py-1.5 focus:outline-none">
                            <option value="string">String</option>
                            <option value="hash">Hash</option>
                            <option value="list">List</option>
                            <option value="set">Set</option>
                        </select>
                    </div>

                    <div>
                        <label for="cmd" class="block text-sm text-gray-300 mb-1">Command</label>
                        <select name="cmd" bind:value={command} class="w-full bg-(--border) rounded-[6px] px-3 py-1.5 focus:outline-none">
                            {#each commands[dataType as keyof typeof commands] || [] as cmd}
                                <option value={cmd.value}>{cmd.label} - {cmd.description}</option>
                            {/each}
                        </select>
                    </div>
                </div>
            </div>

            <div class="bg-(--accent) border border-(--border) rounded-[12px] p-6 space-y-6">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2 text-(--primary)">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z" />
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
                        </svg>
                        <span class="font-medium">Parameters</span>
                    </div>
                    <span class="text-xs bg-(--primary) text-(--dark) px-3 py-1 rounded-full font-medium">{dataType}</span>
                </div>
                
                <div class="space-y-3">
                    <div>
                        <label for="key" class="block text-sm text-gray-300 mb-1">Key *</label>
                        <input 
                            type="text" 
                            name="key"
                            bind:value={key} 
                            placeholder="Enter key name"
                            class="w-full bg-(--border) rounded-[6px] px-3 py-1.5 focus:outline-none"
                        />
                    </div>

                    {#if needsField()}
                        <div>
                            <label for="field" class="block text-sm text-gray-300 mb-2">Field *</label>
                            <input 
                                type="text" 
                                name="field"
                                bind:value={field} 
                                placeholder="Enter field name"
                                class="w-full bg-(--border) rounded-[6px] px-3 py-1.5 focus:outline-none"
                            />
                        </div>
                    {/if}

                    {#if needsValue()}
                        <div>
                            <label for="value" class="block text-sm text-gray-300 mb-2">Value *</label>
                            <input 
                                type="text" 
                                name="value"
                                bind:value={value} 
                                placeholder="Enter value"
                                class="w-full bg-(--border) rounded-[6px] px-3 py-1.5 focus:outline-none"
                            />
                        </div>
                    {/if}

                    {#if needsTtl()}
                        <div>
                            <label for="ttl" class="block text-sm text-gray-300 mb-2">TTL (seconds) *</label>
                            <input 
                                type="number" 
                                name="ttl"
                                bind:value={ttl} 
                                min="1"
                                placeholder="Enter TTL in seconds"
                                class="w-full bg-(--border) rounded-[6px] px-3 py-1.5 focus:outline-none"
                            />
                        </div>
                    {/if}

                    {#if needsRange()}
                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label for="start" class="block text-sm text-gray-300 mb-2">Start Index</label>
                                <input 
                                    type="number" 
                                    name="start"
                                    bind:value={start} 
                                    placeholder="Start index"
                                    class="w-full bg-(--border) rounded-[6px] px-3 py-1.5 focus:outline-none"
                                />
                            </div>
                            <div>
                                <label for="end" class="block text-sm text-gray-300 mb-2">End Index</label>
                                <input 
                                    type="number" 
                                    name="end"
                                    bind:value={end} 
                                    placeholder="End index (-1 for end)"
                                    class="w-full bg-(--border) rounded-[6px] px-3 py-1.5 focus:outline-none"
                                />
                            </div>
                        </div>
                    {/if}
                </div>
            </div>
        </div>

        <!-- Execute Button -->
        <div class="flex justify-center">
            <button 
                onclick={runCommand}
                disabled={loading || !isFormValid()}
                class="bg-(--primary) border border-(--primary-border) text-(--dark) px-6 py-3 rounded-[6px] font-medium hover:bg-(--primary-hover) transition-colors disabled:bg-gray-700 disabled:text-gray-500 disabled:cursor-not-allowed disabled:border-gray-600 cursor-pointer"
            >
                {loading ? 'Executing...' : 'Execute Command'}
            </button>
        </div>

        <!-- Response -->
        <div class="bg-(--accent) border border-(--border) rounded-[12px] p-6">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-medium text-(--primary)">Response</h3>
                {#if lastCommand}
                    <code class="text-sm bg-(--border) px-2 py-1 rounded text-(--primary)">{lastCommand}</code>
                {/if}
            </div>
            
            <div class="bg-(--dark) text-(--light) rounded-[6px] p-3 min-h-[100px] font-mono text-sm">
                {#if loading}
                    <div class="text-(--primary) flex items-center gap-2">
                        <div class="animate-spin w-4 h-4 border-2 border-(--primary) border-t-transparent rounded-full"></div>
                        Executing command...
                    </div>
                {:else if response}
                    <div class="whitespace-pre-wrap {response.type === 'success' ? 'text-(--primary)' : 'text-red-400'}">
                        {response.content}
                    </div>
                {:else}
                    <div class="text-(--light)">
                        No response yet. Execute a command to see results.
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>