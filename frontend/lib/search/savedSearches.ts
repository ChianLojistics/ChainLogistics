export type SavedSearch<TFilters extends Record<string, unknown>> = {
  id: string
  name: string
  filters: TFilters
  createdAt: number
}

export function loadSavedSearches<TFilters extends Record<string, unknown>>(
  storageKey: string
): SavedSearch<TFilters>[] {
  if (typeof window === "undefined") return []

  const raw = localStorage.getItem(storageKey)
  if (!raw) return []

  try {
    const parsed = JSON.parse(raw) as unknown
    if (!Array.isArray(parsed)) return []
    return parsed as SavedSearch<TFilters>[]
  } catch {
    return []
  }
}

export function persistSavedSearches<TFilters extends Record<string, unknown>>(
  storageKey: string,
  items: SavedSearch<TFilters>[]
) {
  if (typeof window === "undefined") return
  localStorage.setItem(storageKey, JSON.stringify(items))
}
