import { Component, createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import { Search, ChevronDown, Check } from 'lucide-solid';

interface SearchableSelectOption {
  value: string;
  label: string;
  sublabel?: string;
  badge?: string;
}

export interface SearchableSelectProps {
  value: string;
  options: SearchableSelectOption[];
  onChange: (value: string) => void;
  placeholder?: string;
  searchPlaceholder?: string;
  class?: string;
  buttonClass?: string;
  dropdownClass?: string;
  disabled?: boolean;
}

export const SearchableSelect: Component<SearchableSelectProps> = (props) => {
  const [isOpen, setIsOpen] = createSignal(false);
  const [search, setSearch] = createSignal('');
  let containerRef: HTMLDivElement | undefined;
  let searchInputRef: HTMLInputElement | undefined;

  const selectedOption = () => props.options.find((o) => o.value === props.value);

  const filteredOptions = () => {
    const q = search().trim().toLowerCase();
    if (!q) return props.options;
    return props.options.filter(
      (o) =>
        o.label.toLowerCase().includes(q) ||
        o.value.toLowerCase().includes(q) ||
        (o.sublabel && o.sublabel.toLowerCase().includes(q))
    );
  };

  const handleToggle = () => {
    if (props.disabled) return;
    const next = !isOpen();
    setIsOpen(next);
    if (next) {
      setSearch('');
      setTimeout(() => searchInputRef?.focus(), 50);
    }
  };

  const handleSelect = (val: string) => {
    props.onChange(val);
    setIsOpen(false);
  };

  const handleClickOutside = (e: MouseEvent) => {
    if (containerRef && !containerRef.contains(e.target as Node)) {
      setIsOpen(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      setIsOpen(false);
    }
  };

  createEffect(() => {
    if (isOpen()) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleKeyDown);
    } else {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    }
  });

  onCleanup(() => {
    document.removeEventListener('mousedown', handleClickOutside);
    document.removeEventListener('keydown', handleKeyDown);
  });

  return (
    <div ref={containerRef} class={`relative inline-block ${props.class || 'w-full'}`}>
      {/* Trigger Button */}
      <button
        type="button"
        disabled={props.disabled}
        onClick={handleToggle}
        class={
          props.buttonClass ||
          'w-full flex items-center justify-between gap-2 px-3 py-2 bg-theme-card border border-theme-border text-xs text-theme-text font-mono hover:border-theme-cyan/50 focus:outline-none transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
        }
      >
        <span class="truncate text-left">
          {selectedOption()?.label || props.placeholder || props.value || 'Select option...'}
        </span>
        <ChevronDown size={14} class={`shrink-0 text-theme-text/50 transition-transform ${isOpen() ? 'rotate-180' : ''}`} />
      </button>

      {/* Dropdown Menu */}
      <Show when={isOpen()}>
        <div
          class={
            props.dropdownClass ||
            'absolute left-0 mt-1.5 w-full min-w-[240px] max-w-[420px] bg-[#12161f] border border-white/10 rounded-lg shadow-2xl z-50 overflow-hidden flex flex-col max-h-72 animate-in fade-in zoom-in-95 duration-100'
          }
        >
          {/* Search Box */}
          <div class="p-2 border-b border-white/10 flex items-center gap-2 bg-black/20 shrink-0">
            <Search size={13} class="text-white/70 shrink-0 ml-1" />
            <input
              ref={searchInputRef}
              type="text"
              value={search()}
              onInput={(e) => setSearch(e.currentTarget.value)}
              placeholder={props.searchPlaceholder || 'Search models / options...'}
              class="w-full bg-transparent text-xs text-white placeholder-white/40 focus:outline-none font-mono py-1"
            />
            <Show when={search()}>
              <button
                type="button"
                onClick={() => setSearch('')}
                class="text-xs text-white/70 hover:text-white px-1 font-mono"
              >
                ×
              </button>
            </Show>
          </div>

          {/* Options List */}
          <div class="overflow-y-auto flex-1 p-1 custom-scrollbar space-y-0.5">
            <For
              each={filteredOptions()}
              fallback={
                <div class="px-3 py-4 text-center text-xs text-white/70 font-mono">
                  No matching options found
                </div>
              }
            >
              {(opt) => {
                const isSelected = () => opt.value === props.value;
                return (
                  <button
                    type="button"
                    onClick={() => handleSelect(opt.value)}
                    class={`w-full flex items-center justify-between gap-2 px-2.5 py-1.5 rounded text-left text-xs font-mono transition-colors ${
                      isSelected()
                        ? 'bg-indigo-600 text-white font-medium shadow-sm'
                        : 'text-white/80 hover:bg-white/10 hover:text-white'
                    }`}
                  >
                    <div class="truncate flex-1">
                      <div class="truncate flex items-center gap-1.5">
                        <span class="truncate">{opt.label}</span>
                        <Show when={opt.badge}>
                          <span class="text-2xs px-1.5 py-0.2 rounded bg-white/10 text-white/60 font-sans">
                            {opt.badge}
                          </span>
                        </Show>
                      </div>
                      <Show when={opt.sublabel}>
                        <div class="text-2xs text-white/70 truncate">{opt.sublabel}</div>
                      </Show>
                    </div>
                    <Show when={isSelected()}>
                      <Check size={13} class="shrink-0 text-white" />
                    </Show>
                  </button>
                );
              }}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
};
