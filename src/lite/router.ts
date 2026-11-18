/*
 * Minimal stand-in for vue-router (aliased as 'vue-router' in vite.config.ts
 * and tsconfig.json) to keep the firmware-embedded page small.
 *
 * Supports exactly what this UI uses:
 *   - createRouter({ history: createWebHistory(), routes }) with nested,
 *     named routes and one catch-all ('/:pathMatch(.*)*')
 *   - router.push({ name }), router.beforeEach(async (to) => redirect | void)
 *   - <RouterView> (nested by depth), <RouterLink :to="{ name }">
 *     (adds 'router-link-active' when the target is on the current route chain)
 *   - useRouter(), useRoute().name
 * HTML5 history mode: the firmware serves index.html for every GET path.
 */
import { defineComponent, h, inject, provide, shallowRef, type App, type Component, type PropType } from 'vue';

export interface RouteRecordRaw {
  path: string;
  name?: string;
  component: Component;
  children?: RouteRecordRaw[];
}

export interface RouteLocationNamedRaw {
  name: string;
}

interface RouteRecord {
  name?: string;
  fullPath: string;
  component: Component;
  parent?: RouteRecord;
  catchAll: boolean;
}

export interface RouteLocation {
  name?: string;
  path: string;
  matched: RouteRecord[];        // root -> leaf
}

type Guard = (to: RouteLocation) => unknown;

export interface Router {
  push(to: RouteLocationNamedRaw): Promise<void>;
  beforeEach(guard: Guard): void;
  install(app: App): void;
}

const EMPTY: RouteLocation = { path: '', matched: [] };
const current = shallowRef<RouteLocation>(EMPTY);
const DEPTH_KEY = Symbol('routerViewDepth');
let activeRouter: Router;
let recordByName: (name: string) => RouteRecord | undefined = () => undefined;

export function createWebHistory() {
  return {};
}

function joinPath(base: string, path: string): string {
  if (path.startsWith('/')) return path;
  return (base.endsWith('/') ? base : base + '/') + path;
}

function normalize(path: string): string {
  return path.length > 1 ? path.replace(/\/+$/, '') : path;
}

export function createRouter(options: { history: object; routes: readonly RouteRecordRaw[] }): Router {
  const records: RouteRecord[] = [];
  const add = (raw: RouteRecordRaw, parent?: RouteRecord) => {
    const rec: RouteRecord = {
      name: raw.name,
      fullPath: normalize(joinPath(parent ? parent.fullPath : '/', raw.path)),
      component: raw.component,
      parent,
      catchAll: raw.path.includes(':pathMatch')
    };
    records.push(rec);
    raw.children?.forEach((child) => add(child, rec));
  };
  options.routes.forEach((r) => add(r));

  const guards: Guard[] = [];

  const toLocation = (rec: RouteRecord, path: string): RouteLocation => {
    const matched: RouteRecord[] = [];
    for (let r: RouteRecord | undefined = rec; r; r = r.parent) matched.unshift(r);
    return { name: rec.name, path, matched };
  };

  const byName = (name: string) => records.find((r) => r.name === name);
  const byPath = (path: string) => {
    const p = normalize(path);
    const rec = records.find((r) => !r.catchAll && r.fullPath === p) || records.find((r) => r.catchAll);
    return rec ? toLocation(rec, rec.catchAll ? p : rec.fullPath) : EMPTY;
  };

  // Runs guards (following redirects) and commits the final location.
  const navigate = async (to: RouteLocation, mode: 'push' | 'replace') => {
    for (let hops = 0; hops < 5; hops++) {
      let redirect: RouteLocationNamedRaw | undefined;
      for (const guard of guards) {
        const r = await guard(to);
        if (r && typeof r === 'object' && 'name' in r) {
          redirect = r as RouteLocationNamedRaw;
          break;
        }
      }
      if (!redirect) break;
      const rec = byName(redirect.name);
      if (!rec) break;
      to = toLocation(rec, rec.fullPath);
    }
    if (location.pathname !== to.path) {
      if (mode === 'push') history.pushState(null, '', to.path);
      else history.replaceState(null, '', to.path);
    }
    current.value = to;
  };

  const router: Router = {
    push(to) {
      const rec = byName(to.name);
      if (!rec) {
        console.warn('route not found:', to.name);
        return Promise.resolve();
      }
      return navigate(toLocation(rec, rec.fullPath), 'push');
    },
    beforeEach(guard) {
      guards.push(guard);
    },
    install(app) {
      app.component('RouterView', RouterView);
      app.component('RouterLink', RouterLink);
      window.addEventListener('popstate', () => navigate(byPath(location.pathname), 'replace'));
      // Run after install so guards registered at module load are in place.
      Promise.resolve().then(() => navigate(byPath(location.pathname), 'replace'));
    }
  };
  activeRouter = router;
  recordByName = byName;
  return router;
}

export function useRouter(): Router {
  return activeRouter;
}

export function useRoute() {
  return {
    get name() { return current.value.name; },
    get path() { return current.value.path; }
  };
}

export const RouterView = defineComponent({
  name: 'RouterView',
  setup() {
    const depth = inject<number>(DEPTH_KEY, 0);
    provide(DEPTH_KEY, depth + 1);
    return () => {
      const rec = current.value.matched[depth];
      return rec ? h(rec.component) : null;
    };
  }
});

export const RouterLink = defineComponent({
  name: 'RouterLink',
  props: {
    to: { type: Object as PropType<RouteLocationNamedRaw>, required: true }
  },
  setup(props, { slots }) {
    return () => {
      const rec = recordByName(props.to.name);
      const active = current.value.matched.some((r) => r.name === props.to.name);
      return h('a', {
        href: rec ? rec.fullPath : '#',
        class: active ? 'router-link-active' : undefined,
        onClick: (e: MouseEvent) => {
          if (e.ctrlKey || e.metaKey || e.shiftKey || e.button !== 0) return;
          e.preventDefault();
          activeRouter.push(props.to);
        }
      }, slots.default?.());
    };
  }
});
