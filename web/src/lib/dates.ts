export function pickDate(dates:string[], today:string){ return dates.includes(today)?today:dates[dates.length-1]; }
