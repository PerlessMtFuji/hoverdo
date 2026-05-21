<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  type Variant = 'primary' | 'ghost' | 'danger';
  type Size = 'sm' | 'md';

  type Props = HTMLButtonAttributes & {
    variant?: Variant;
    size?: Size;
    children?: Snippet;
  };

  let {
    variant = 'ghost',
    size = 'md',
    children,
    class: extra = '',
    type = 'button',
    ...rest
  }: Props = $props();

  const sizeCls: Record<Size, string> = {
    sm: 'h-7 px-2 text-xs',
    md: 'h-8 px-3 text-sm'
  };

  const variantCls: Record<Variant, string> = {
    primary:
      'bg-accent text-accent-fg shadow-hd-sm hover:brightness-105 active:brightness-95',
    ghost:
      'bg-transparent text-text-1 hover:bg-surface-2',
    danger:
      'bg-transparent text-danger hover:bg-danger/10'
  };
</script>

<button
  {type}
  class="inline-flex items-center justify-center gap-1.5 rounded-control font-medium
    transition-[background,filter,transform] duration-150 ease-out
    focus-visible:outline-2 focus-visible:outline-offset-2
    active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-50
    {sizeCls[size]} {variantCls[variant]} {extra}"
  {...rest}
>
  {#if children}{@render children()}{/if}
</button>
