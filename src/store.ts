import { writable } from 'svelte/store'

export const linked_paths = writable([] as LinkedPath[])
export const networks = writable([] as Network[])
