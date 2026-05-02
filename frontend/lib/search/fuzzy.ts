export type FuzzyMatch = {
  score: number
  matched: boolean
}

export function fuzzyScore(query: string, text: string): FuzzyMatch {
  const q = query.trim().toLowerCase()
  const t = text.trim().toLowerCase()

  if (!q) return { score: 0, matched: true }
  if (!t) return { score: 0, matched: false }

  let qi = 0
  let score = 0
  let consecutive = 0

  for (let ti = 0; ti < t.length && qi < q.length; ti += 1) {
    const tc = t[ti]!
    const qc = q[qi]!

    if (tc === qc) {
      const isWordStart = ti === 0 || /[^a-z0-9]/.test(t[ti - 1]!)
      score += 5
      if (isWordStart) score += 6

      consecutive += 1
      score += Math.min(10, consecutive * 2)

      qi += 1
      continue
    }

    consecutive = 0
  }

  const matched = qi === q.length
  if (!matched) return { score: 0, matched: false }

  const lengthPenalty = Math.max(0, t.length - q.length)
  score -= Math.min(15, Math.floor(lengthPenalty / 8))

  return { score: Math.max(1, score), matched: true }
}

export function fuzzyIncludes(query: string, text: string) {
  return fuzzyScore(query, text).matched
}
