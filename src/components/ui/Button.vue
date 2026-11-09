<script setup lang="ts">
import { cn } from "@/utils/cn";

type Variant = "default" | "primary" | "ghost" | "danger" | "subtle";
const props = withDefaults(defineProps<{ variant?: Variant; size?: "sm" | "md"; type?: "button" | "submit" }>(), { variant: "default", size: "md", type: "button" });

const variants: Record<Variant, string> = {
  default: "bg-raised hover:bg-line/80 text-fg border border-line",
  primary: "bg-accent text-accent-fg hover:brightness-110 border border-accent",
  ghost: "hover:bg-raised text-muted hover:text-fg",
  subtle: "bg-raised/60 hover:bg-raised text-fg",
  danger: "bg-danger text-white hover:brightness-110 border border-danger",
};
</script>

<template>
  <button
    :type="props.type"
    :class="
      cn(
        'inline-flex items-center justify-center gap-1.5 whitespace-nowrap rounded-md font-medium transition-colors disabled:pointer-events-none disabled:opacity-40',
        props.size === 'sm' ? 'h-7 px-2.5 text-xs' : 'h-8 px-3 text-[13px]',
        variants[props.variant],
      )
    "
  >
    <slot />
  </button>
</template>
