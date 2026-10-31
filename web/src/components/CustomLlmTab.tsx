import { Component, For, Show, createSignal, onMount } from 'solid-js';
import { CustomLlmAccount, customLlmService } from '../services/customLlmService';

const emptyForm = (): CustomLlmAccount => ({ prefix: '', name: '', base_url: '', api_key: '', enabled: true });
const fieldClass = 'mt-1.5 w-full border border-theme-border bg-theme-bg px-3 py-2.5 text-sm text-theme-text outline-none transition-colors placeholder:text-theme-text/35 focus:border-theme-cyan';

export const CustomLlmTab: Component<{ showToast: (message: string) => void }> = (props) => {
  const [accounts, setAccounts] = createSignal<CustomLlmAccount[]>([]);
  const [form, setForm] = createSignal<CustomLlmAccount>(emptyForm());
  const [editing, setEditing] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  let formRef: HTMLFormElement | undefined;

  const startEdit = (account: CustomLlmAccount) => {
    setEditing(account.prefix);
    setForm({
      prefix: account.prefix,
      name: account.name,
      base_url: account.base_url,
      api_key: account.api_key,
      enabled: account.enabled,
    });
    setError('');
    formRef?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  };

  const load = async () => {
    try {
      setAccounts(await customLlmService.getAccounts());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };
  onMount(load);

  const save = async (event: SubmitEvent) => {
    event.preventDefault();
    const value = form();
    const prefix = value.prefix.trim().toLowerCase();
    if (!/^[a-z0-9_-]+$/.test(prefix)) { setError('Prefix may contain only letters, numbers, underscores, and hyphens.'); return; }
    if (['ag', 'cx', 'cc'].includes(prefix)) { setError('The prefixes ag, cx, and cc are reserved for built-in providers.'); return; }
    if (!value.name.trim() || !value.base_url.trim()) { setError('Account name and Base URL are required.'); return; }
    if (!value.api_key.trim()) { setError('An API key is required.'); return; }
    if (accounts().some(a => a.prefix.toLowerCase() === prefix && a.prefix.toLowerCase() !== editing()?.toLowerCase())) {
      setError('That prefix is already in use.');
      return;
    }
    setBusy(true);
    setError('');
    try {
      let next: CustomLlmAccount[];
      const currentEditing = editing();
      const updatedAccount: CustomLlmAccount = {
        prefix,
        name: value.name.trim(),
        base_url: value.base_url.trim(),
        api_key: value.api_key.trim(),
        enabled: value.enabled ?? true,
      };

      if (currentEditing) {
        next = accounts().map(a => a.prefix === currentEditing ? updatedAccount : a);
      } else {
        next = [...accounts(), updatedAccount];
      }

      await customLlmService.saveAccounts(next);
      setForm(emptyForm());
      setEditing(null);
      await load();
      props.showToast(currentEditing ? 'Account updated.' : 'Account registered.');
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const toggleEnabled = async (account: CustomLlmAccount) => {
    setBusy(true);
    setError('');
    try {
      const next = accounts().map(item => item.prefix === account.prefix ? { ...item, enabled: !item.enabled } : item);
      await customLlmService.saveAccounts(next);
      await load();
      props.showToast(`${account.name} ${account.enabled ? 'disabled' : 'enabled'}.`);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (prefix: string) => {
    if (!window.confirm(`Delete the ${prefix} account?`)) return;
    try {
      await customLlmService.deleteAccount(prefix);
      await load();
      props.showToast('Account deleted.');
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return <section class="space-y-6 animate-fade-in text-theme-text select-text">
    <header class="border border-theme-border bg-gradient-to-r from-theme-card via-theme-surface to-theme-bg p-5 shadow-sm sm:p-6">
      <h1 class="font-heading text-xl font-bold tracking-tight sm:text-2xl">Custom OpenAI-Compatible</h1>
      <p class="mt-2 max-w-3xl text-xs leading-relaxed text-theme-text/60 sm:text-sm">
        Register any OpenAI-compatible endpoint with your own prefix. Reference models as <code class="border border-theme-border/60 bg-theme-bg px-1.5 py-0.5 font-mono text-theme-text/80">prefix/model-id</code>. The prefixes <code>ag</code>, <code>cx</code>, and <code>cc</code> are reserved.
      </p>
    </header>

    <Show when={error()}><div role="alert" class="border border-red-500/40 bg-red-500/10 p-3 text-sm text-red-300">{error()}</div></Show>

    <form ref={formRef} onSubmit={save} class={`space-y-5 border bg-theme-card p-5 shadow-sm sm:p-6 transition-all ${editing() ? 'border-theme-cyan/60 ring-1 ring-theme-cyan/30' : 'border-theme-border'}`}>
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-sm font-bold text-theme-text">{editing() ? `Edit provider (${editing()})` : 'Register a provider account'}</h2>
          <p class="mt-1 text-xs text-theme-text/50">Enter the endpoint credentials used for requests to this provider.</p>
        </div>
        <Show when={editing()}>
          <span class="rounded bg-theme-cyan/10 px-2.5 py-1 text-[11px] font-bold text-theme-cyan border border-theme-cyan/30">Editing Active</span>
        </Show>
      </div>
      <div class="grid gap-4 md:grid-cols-2">
        <label class="block text-xs font-semibold text-theme-text/80">Prefix
          <input required value={form().prefix} onInput={e => setForm({ ...form(), prefix: e.currentTarget.value })} placeholder="e.g. 9r or or" class={fieldClass} />
        </label>
        <label class="block text-xs font-semibold text-theme-text/80">Account name
          <input required value={form().name} onInput={e => setForm({ ...form(), name: e.currentTarget.value })} placeholder="e.g. 9router or OpenRouter" class={fieldClass} />
        </label>
        <label class="block text-xs font-semibold text-theme-text/80 md:col-span-2">Base URL
          <input required type="url" value={form().base_url} onInput={e => setForm({ ...form(), base_url: e.currentTarget.value })} placeholder="e.g. http://localhost:20128/v1 or https://openrouter.ai/api/v1" class={fieldClass} />
        </label>
        <label class="block text-xs font-semibold text-theme-text/80 md:col-span-2">API key
          <input type="password" value={form().api_key} onInput={e => setForm({ ...form(), api_key: e.currentTarget.value })} placeholder="Enter API key" class={fieldClass} />
        </label>
      </div>
      <div class="flex flex-wrap gap-2 border-t border-theme-border pt-4">
        <button type="submit" disabled={busy()} class="bg-theme-cyan px-4 py-2.5 text-xs font-bold text-black transition-opacity hover:opacity-80 disabled:opacity-50">{busy() ? 'Saving…' : editing() ? 'Save changes' : 'Add account'}</button>
        <Show when={editing()}><button type="button" onClick={() => { setEditing(null); setForm(emptyForm()); setError(''); }} class="border border-theme-border bg-theme-bg px-4 py-2.5 text-xs font-semibold text-theme-text/80 transition-colors hover:border-theme-text/40">Cancel</button></Show>
      </div>
    </form>

    <div class="space-y-3">
      <div class="flex items-center justify-between border-b border-theme-border pb-2">
        <h2 class="text-sm font-bold text-theme-text">Registered accounts</h2>
        <span class="text-xs text-theme-text/50">{accounts().length} total</span>
      </div>
      <Show when={accounts().length > 0} fallback={<div class="border border-theme-border/60 bg-theme-card p-8 text-center"><p class="text-sm font-semibold text-theme-text/70">No custom accounts yet</p><p class="mt-1 text-xs text-theme-text/45">Add an account above to get started.</p></div>}>
        <For each={accounts()}>{account => <article class={`flex flex-wrap items-center justify-between gap-4 border bg-theme-card p-4 shadow-sm transition-all sm:p-5 ${editing() === account.prefix ? 'border-theme-cyan/60 ring-1 ring-theme-cyan/20' : 'border-theme-border hover:border-theme-border/80'}`}>
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2 text-sm font-bold text-theme-text">{account.name}<code class="border border-theme-border/60 bg-theme-bg px-1.5 py-0.5 font-mono text-xs font-normal text-theme-cyan">{account.prefix}/</code></div>
            <div class="mt-2 flex flex-wrap items-center gap-2">
              <span class={`border px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide ${account.enabled ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300' : 'border-theme-border bg-theme-bg text-theme-text/45'}`}>{account.enabled ? 'Enabled' : 'Disabled'}</span>
              <span class="break-all font-mono text-xs text-theme-text/50">{account.base_url}</span>
            </div>
          </div>
          <div class="flex gap-2">
            <button type="button" disabled={busy()} onClick={() => toggleEnabled(account)} class={`border px-3 py-2 text-xs font-semibold transition-colors disabled:opacity-50 ${account.enabled ? 'border-theme-border bg-theme-bg text-theme-text/80 hover:border-amber-400 hover:text-amber-300' : 'border-emerald-500/30 bg-emerald-500/5 text-emerald-300 hover:bg-emerald-500/10'}`}>{account.enabled ? 'Disable' : 'Enable'}</button>
            <button type="button" onClick={() => startEdit(account)} class={`border px-3 py-2 text-xs font-semibold transition-colors ${editing() === account.prefix ? 'border-theme-cyan bg-theme-cyan/10 text-theme-cyan' : 'border-theme-border bg-theme-bg text-theme-text/80 hover:border-theme-cyan hover:text-theme-cyan'}`}>{editing() === account.prefix ? 'Editing' : 'Edit'}</button>
            <button type="button" onClick={() => remove(account.prefix)} class="border border-red-500/30 bg-red-500/5 px-3 py-2 text-xs font-semibold text-red-300 transition-colors hover:border-red-500/60 hover:bg-red-500/10">Delete</button>
          </div>
        </article>}</For>
      </Show>
    </div>
  </section>;
};
