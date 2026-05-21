export function pickDate(dates: readonly string[], today: string): string {
  if (dates.includes(today)) return today;
  return [...dates].sort().at(-1) ?? today;
}

export function explicitDate(date: string): string {
  return date;
}
