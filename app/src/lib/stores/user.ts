import { writable } from "svelte/store"

function persistStore<T>(key: string, initial: T) {
  const storedValue = localStorage.getItem(key)
  const data: any = storedValue ? JSON.parse(storedValue) : initial
  const store = writable<T>(data)

  store.subscribe(value => {
    localStorage.setItem(key, JSON.stringify(value))
  })

  return store
}

export const showUnsupported = persistStore("show-unsupported", false)
