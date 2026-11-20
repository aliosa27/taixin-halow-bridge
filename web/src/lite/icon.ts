/*
 * Minimal stand-in for @vicons/utils (aliased in vite.config.ts and
 * tsconfig.json): same markup (<span class="xicon">) and props, but the
 * styles live in assets/style.css instead of being injected via css-render.
 */
import { defineComponent, h, inject, provide, type PropType } from 'vue';

const CONFIG_KEY = Symbol('iconConfig');

const iconProps = {
  color: String,
  size: [Number, String] as PropType<number | string>,
  tag: String
};

type IconConfig = { color?: string; size?: number | string; tag?: string };

export const IconConfigProvider = defineComponent({
  name: 'IconConfigProvider',
  props: iconProps,
  setup(props, { slots }) {
    provide(CONFIG_KEY, props);
    return () => slots.default?.();
  }
});

export const Icon = defineComponent({
  name: 'Icon',
  props: iconProps,
  setup(props, { slots }) {
    const cfg = inject<IconConfig | null>(CONFIG_KEY, null);
    return () => {
      const size = props.size ?? cfg?.size;
      return h(props.tag ?? cfg?.tag ?? 'span', {
        class: 'xicon',
        style: {
          color: props.color ?? cfg?.color,
          fontSize: size === undefined ? undefined : /^\d+$/.test(String(size)) ? `${size}px` : size
        }
      }, slots.default?.());
    };
  }
});
