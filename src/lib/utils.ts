import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function mergeLessThanPercent(data: { label: string, total: number }[], threshold = 1) {
  const total = data.reduce((acc, { total }) => acc + total, 0)
  
  const others = {
    label: 'Others',
    total: 0
  }

  const result = data.reduce((acc, item) => {
    const percentage = (item.total / total) * 100
    
    if (percentage < threshold) {
      others.total += item.total
      return acc
    }
    
    acc.push(item)
    return acc
  }, [] as typeof data)

  if (others.total > 0) {
    result.push(others)
  }

  return result
}